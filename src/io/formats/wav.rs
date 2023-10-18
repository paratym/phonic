use crate::{
    io::{
        EncodedStreamSpecBuilder, FormatDataBuilder, FormatIdentifiers, FormatReadResult,
        FormatReader, MediaSource, SyphonCodec,
    },
    SampleFormat, SyphonError,
};
use std::io::{Read, SeekFrom};

pub static WAV_FORMAT_IDENTIFIERS: FormatIdentifiers = FormatIdentifiers {
    file_extensions: &["wav", "wave"],
    mime_types: &["audio/vnd.wave", "audio/x-wav", "audio/wav", "audio/wave"],
    markers: &[b"RIFF", b"WAVE"],
};

#[derive(Clone, Copy)]
pub struct WavCodecKey(pub u16);

impl TryFrom<WavCodecKey> for SyphonCodec {
    type Error = SyphonError;

    fn try_from(WavCodecKey(id): WavCodecKey) -> Result<Self, Self::Error> {
        match id {
            1 | 3 => Ok(SyphonCodec::Pcm),
            _ => Err(SyphonError::Unsupported),
        }
    }
}

pub struct WavReader {
    source: Box<dyn MediaSource>,
    data: FormatDataBuilder,
    media_bounds: (u64, u64),
    source_pos: u64,
}

impl WavReader {
    pub fn new(mut source: Box<dyn MediaSource>) -> Self {
        let source_pos = source.stream_position().unwrap_or(0);

        Self {
            source,
            data: FormatDataBuilder::new(),
            media_bounds: (0, 0),
            source_pos,
        }
    }

    fn stream_spec_mut(&mut self) -> &mut EncodedStreamSpecBuilder {
        let tracks = &mut self.data.tracks;
        if tracks.is_empty() {
            tracks.push(EncodedStreamSpecBuilder::new());
        }

        return tracks.first_mut().unwrap();
    }

    fn media_pos(&self) -> u64 {
        self.source_pos
            .max(self.media_bounds.0)
            .min(self.media_bounds.1)
    }

    fn parse_fmt_chunk(&mut self, buf: &mut [u8]) -> Result<(), SyphonError> {
        let chunk_len = buf.len();
        if chunk_len != 16 && chunk_len != 18 && chunk_len != 40 {
            return Err(SyphonError::MalformedData);
        }

        let encoded_stream_spec = self.stream_spec_mut();
        let codec_key = u16::from_le_bytes(buf[0..2].try_into().unwrap());
        encoded_stream_spec.codec_key = WavCodecKey(codec_key).try_into().ok();

        let stream_spec = &mut encoded_stream_spec.decoded_spec;
        stream_spec.n_channels = Some(u16::from_le_bytes(buf[2..4].try_into().unwrap()));
        stream_spec.sample_rate = Some(u32::from_le_bytes(buf[4..8].try_into().unwrap()));
        stream_spec.block_size = Some(u16::from_le_bytes(buf[12..14].try_into().unwrap()) as usize);

        let bits_per_sample = u16::from_le_bytes(buf[14..16].try_into().unwrap());
        stream_spec.sample_format = match (codec_key, bits_per_sample) {
            (1, 8) => SampleFormat::U8.into(),
            (1, 16) => SampleFormat::I16.into(),
            (1, 32) => SampleFormat::I32.into(),
            (1, 64) => SampleFormat::I64.into(),
            (3, 32) => SampleFormat::F32.into(),
            (3, 64) => SampleFormat::F64.into(),
            _ => None,
        };

        Ok(())
    }

    fn parse_fact_chunk(&mut self, buf: &mut [u8]) -> Result<(), SyphonError> {
        if buf.len() != 4 {
            return Err(SyphonError::MalformedData);
        }

        let stream_spec = &mut self.stream_spec_mut().decoded_spec;
        if let Some(n_channels) = stream_spec.n_channels {
            stream_spec.n_frames = Some(
                u32::from_le_bytes(buf[0..4].try_into().unwrap()) as usize / n_channels as usize,
            );
        }

        Ok(())
    }

    fn get_chunk_parser(
        chunk_id: &[u8],
    ) -> Option<fn(&mut Self, &mut [u8]) -> Result<(), SyphonError>> {
        match chunk_id {
            b"fmt " => Some(Self::parse_fmt_chunk),
            b"fact" => Some(Self::parse_fact_chunk),
            _ => None,
        }
    }
}

impl FormatReader for WavReader {
    fn try_format(&mut self) -> Result<(), SyphonError> {
        let mut buf = [0u8; 40];

        self.source.read_exact(&mut buf[0..12])?;
        self.source_pos += 12;
        if &buf[0..4] != b"RIFF" || &buf[8..12] != b"WAVE" {
            return Err(SyphonError::MalformedData);
        }

        loop {
            self.source.read_exact(&mut buf[..8])?;
            self.source_pos += 8;

            let chunk_id = &buf[0..4];
            let chunk_len = u32::from_le_bytes(buf[4..8].try_into().unwrap()) as usize;

            if chunk_id == b"data" {
                self.media_bounds = (self.source_pos, self.source_pos + chunk_len as u64);
                self.stream_spec_mut().byte_len = Some(chunk_len);
                return Ok(());
            }

            if let Some(parser) = Self::get_chunk_parser(chunk_id) {
                if chunk_len as usize > buf.len() {
                    return Err(SyphonError::MalformedData);
                }

                self.source.read_exact(&mut buf[..chunk_len])?;
                self.source_pos += chunk_len as u64;
                parser(self, &mut buf[..chunk_len])?;
            } else {
                self.source_pos = self.source.seek(SeekFrom::Current(chunk_len as i64))?;
            }
        }
    }

    fn format_data(&self) -> &FormatDataBuilder {
        &self.data
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError> {
        if self.source_pos < self.media_bounds.0 {
            return Err(SyphonError::NotReady);
        } else if self.source_pos >= self.media_bounds.1 {
            return Err(SyphonError::Empty);
        }

        let mut len = buf
            .len()
            .min((self.media_bounds.1 - self.source_pos) as usize);
        let block_size = self.stream_spec_mut().block_size.unwrap_or(1);
        len -= len % block_size;

        let n_bytes = self.source.read(&mut buf[..len])?;
        self.source_pos += n_bytes as u64;

        if n_bytes % block_size != 0 {
            todo!();
        }

        Ok(FormatReadResult {
            n_bytes,
            track_i: 0,
        })
    }

    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError> {
        let new_pos = match offset {
            SeekFrom::Current(offset) if offset == 0 => {
                return Ok(self.media_pos());
            }
            SeekFrom::Current(offset) => self.media_pos() as i64 + offset,
            SeekFrom::Start(offset) => self.media_bounds.0 as i64 + offset as i64,
            SeekFrom::End(offset) => self.media_bounds.1 as i64 + offset,
        };

        if new_pos < 0 {
            return Err(SyphonError::BadRequest);
        }

        let mut new_pos = new_pos as u64;
        if new_pos < self.media_bounds.0 || new_pos >= self.media_bounds.1 {
            return Err(SyphonError::BadRequest);
        }

        let new_media_pos = new_pos - self.media_bounds.0;
        let block_size = self.stream_spec_mut().block_size.unwrap_or(1);
        new_pos -= new_media_pos % block_size as u64;

        self.source_pos = self.source.seek(SeekFrom::Start(new_pos))?;
        Ok(self.media_pos())
    }
}

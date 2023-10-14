use crate::{
    io::{
        FormatIdentifiers, FormatReadResult, FormatReader, MediaSource, StreamSpecBuilder,
        SyphonCodec,
    },
    Endianess, SampleFormat, SyphonError,
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
    stream_spec: StreamSpecBuilder,
    media_bounds: Option<(usize, usize)>,
    source_pos: usize,
}

impl WavReader {
    pub fn new(mut reader: Box<dyn MediaSource>) -> Self {
        let source_pos = reader.stream_position().unwrap_or(0) as usize;

        Self {
            source: reader,
            stream_spec: StreamSpecBuilder::new(),
            media_bounds: None,
            source_pos,
        }
    }

    fn read_riff_header(&mut self, buf: &mut [u8; 12]) -> Result<(), SyphonError> {
        self.source.read_exact(buf)?;
        self.source_pos += buf.len();

        if &buf[0..4] != b"RIFF" || &buf[8..12] != b"WAVE" {
            return Err(SyphonError::MalformedData);
        }

        Ok(())
    }

    fn read_fmt_chunk(&mut self, buf: &mut [u8]) -> Result<(), SyphonError> {
        let chunk_len = buf.len();
        if chunk_len != 16 && chunk_len != 18 && chunk_len != 40 {
            return Err(SyphonError::MalformedData);
        }

        self.source.read_exact(buf)?;
        self.source_pos += chunk_len;

        let codec_key = u16::from_le_bytes(buf[0..2].try_into().unwrap());
        self.stream_spec.codec_key = WavCodecKey(codec_key).try_into().ok();

        self.stream_spec.n_channels = Some(u16::from_le_bytes(buf[2..4].try_into().unwrap()));
        self.stream_spec.sample_rate = Some(u32::from_le_bytes(buf[4..8].try_into().unwrap()));
        self.stream_spec.block_size =
            Some(u16::from_le_bytes(buf[12..14].try_into().unwrap()) as usize);

        let bits_per_sample = u16::from_le_bytes(buf[14..16].try_into().unwrap());
        self.stream_spec.bytes_per_sample = Some(bits_per_sample / 8);
        self.stream_spec.sample_format = match codec_key {
            1 if bits_per_sample == 8 => Some(SampleFormat::Unsigned(Endianess::Little)),
            1 => Some(SampleFormat::Signed(Endianess::Little)),
            3 => Some(SampleFormat::Float(Endianess::Little)),
            _ => None,
        };

        Ok(())
    }

    fn read_fact_chunk(&mut self, buf: &mut [u8; 4]) -> Result<(), SyphonError> {
        self.source.read_exact(buf)?;
        self.source_pos += buf.len();

        if let Some(n_channels) = self.stream_spec.n_channels {
            self.stream_spec.n_frames =
                Some(u32::from_le_bytes(*buf) as usize / n_channels as usize);
        }

        Ok(())
    }
}

impl FormatReader for WavReader {
    fn tracks(&self) -> Box<dyn Iterator<Item = StreamSpecBuilder>> {
        Box::new(std::iter::once(self.stream_spec))
    }

    fn read_headers(&mut self) -> Result<(), SyphonError> {
        let mut buf = [0u8; 40];

        self.read_riff_header(&mut buf[0..12].try_into().unwrap())?;

        loop {
            self.source
                .read_exact(&mut buf[..8])
                .map_err(|e| Into::<SyphonError>::into(e))?;

            self.source_pos += 8;
            let chunk_len = u32::from_le_bytes(buf[4..8].try_into().unwrap()) as usize;

            match &buf[0..4] {
                b"fmt " => {
                    if chunk_len > buf.len() {
                        return Err(SyphonError::MalformedData);
                    }

                    self.read_fmt_chunk(&mut buf[..chunk_len])?
                }
                b"fact" => {
                    if chunk_len != 4 {
                        return Err(SyphonError::MalformedData);
                    }

                    self.read_fact_chunk(&mut buf[..4].try_into().unwrap())?
                }
                b"data" => {
                    if let Ok(pos) = self.source.stream_position() {
                        if pos as usize != self.source_pos {
                            return Err(SyphonError::MalformedData);
                        }
                    }

                    self.media_bounds = Some((self.source_pos, self.source_pos + chunk_len));
                    break;
                }
                _ => {
                    self.source
                        .seek(SeekFrom::Current(chunk_len as i64))
                        .map_err(|e| Into::<SyphonError>::into(e))?;
                }
            }
        }

        Ok(())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError> {
        let bounds = self.media_bounds.ok_or(SyphonError::NotReady)?;
        if self.source_pos < bounds.0 {
            return Err(SyphonError::NotReady);
        } else if self.source_pos >= bounds.1 {
            return Err(SyphonError::Empty);
        }

        let len = buf.len().min(bounds.1 - self.source_pos);
        let n_bytes = self.source.read(&mut buf[..len])?;
        self.source_pos += n_bytes;

        Ok(FormatReadResult {
            n_bytes,
            track_i: 0,
        })
    }

    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError> {
        let bounds = self.media_bounds.ok_or(SyphonError::NotReady)?;
        let new_pos = match offset {
            SeekFrom::Start(offset) => bounds.0 as i64 + offset as i64,
            SeekFrom::End(offset) => bounds.1 as i64 + offset,
            SeekFrom::Current(offset) if offset == 0 => {
                return Ok(self.source.stream_position()?)
            }
            SeekFrom::Current(offset) => self.source.stream_position()? as i64 + offset,
        };

        if new_pos < 0 {
            return Err(SyphonError::BadRequest);
        }

        let new_pos = new_pos as usize;
        if new_pos < bounds.0 || new_pos > bounds.1 {
            return Err(SyphonError::BadRequest);
        }

        self.source.seek(SeekFrom::Start(new_pos as u64))?;
        self.source_pos = new_pos;

        Ok(new_pos as u64)
    }
}

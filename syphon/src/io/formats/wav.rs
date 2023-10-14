use crate::{
    core::{Endianess, SampleFormat, SyphonError},
    io::{
        FormatIdentifiers, FormatReadResult, FormatReader, MediaSource, SyphonCodec,
        TrackDataBuilder,
    },
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
    track_data: TrackDataBuilder<WavCodecKey>,
}

impl WavReader {
    pub fn new(reader: Box<dyn MediaSource>) -> Self {
        Self {
            source: reader,
            track_data: TrackDataBuilder::new(),
        }
    }

    fn read_riff_header(&mut self, buf: &mut [u8; 12]) -> Result<(), ()> {
        self.source.read_exact(buf).map_err(|_| ())?;
        if &buf[0..4] != b"RIFF" || &buf[8..12] != b"WAVE" {
            return Err(());
        }

        Ok(())
    }

    fn read_chunk_header<'a>(&mut self, buf: &'a mut [u8; 8]) -> Result<&'a [u8; 8], ()> {
        self.source.read_exact(buf).map_err(|_| ())?;
        Ok(buf)
    }

    fn read_fmt_chunk<K: TryFrom<WavCodecKey>>(&mut self, buf: &mut [u8]) -> Result<(), ()> {
        match buf.len() {
            16 | 18 | 40 => (),
            _ => return Err(()),
        }

        self.source.read_exact(buf).map_err(|_| ())?;

        let codec_key = u16::from_le_bytes(buf[0..2].try_into().map_err(|_| ())?);
        self.track_data.codec_key = WavCodecKey(codec_key)
            .try_into()
            .map_or(None, |key| Some(key));

        self.track_data.signal_spec.n_channels =
            Some(u16::from_le_bytes(buf[2..4].try_into().map_err(|_| ())?));
        self.track_data.signal_spec.sample_rate =
            Some(u32::from_le_bytes(buf[4..8].try_into().map_err(|_| ())?));
        self.track_data.signal_spec.block_size =
            Some(u16::from_le_bytes(buf[12..14].try_into().map_err(|_| ())?) as usize);

        let bits_per_sample = u16::from_le_bytes(buf[14..16].try_into().map_err(|_| ())?);
        self.track_data.signal_spec.bytes_per_sample = Some(bits_per_sample / 8);
        self.track_data.signal_spec.sample_format = match codec_key {
            1 if bits_per_sample == 8 => Some(SampleFormat::Unsigned(Endianess::Little)),
            1 => Some(SampleFormat::Signed(Endianess::Little)),
            3 => Some(SampleFormat::Float(Endianess::Little)),
            _ => None,
        };

        Ok(())
    }

    fn read_fact_chunk(&mut self, buf: &mut [u8; 4]) -> Result<(), ()> {
        self.source.read_exact(buf).map_err(|_| ())?;
        // let n_samples = u32::from_le_bytes(*buf);

        Ok(())
    }
}

impl FormatReader for WavReader {
    type CodecKey = WavCodecKey;

    fn tracks(&self) -> Box<dyn Iterator<Item = TrackDataBuilder<Self::CodecKey>>> {
        Box::new(std::iter::once(self.track_data))
    }

    fn read_headers(&mut self) -> Result<usize, SyphonError> {
        todo!()
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError> {
        todo!()
    }

    fn seek(&mut self, offset: SeekFrom) -> Result<usize, SyphonError> {
        todo!()
    }
}

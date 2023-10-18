use crate::{
    io::{EncodedStreamReader, EncodedStreamSpec, FormatReader},
    SyphonError,
};
use std::io::{self, Read, Seek, SeekFrom};

pub struct TrackReader {
    reader: Box<dyn FormatReader>,
    stream_spec: EncodedStreamSpec,
    track_i: usize,
}

impl TrackReader {
    pub fn new(reader: Box<dyn FormatReader>, track_i: usize) -> Result<Self, SyphonError> {
        let stream_spec = reader
            .format_data()
            .tracks
            .iter()
            .nth(track_i)
            .ok_or(SyphonError::BadRequest)?
            .try_build()?;

        Ok(Self {
            reader,
            stream_spec,
            track_i,
        })
    }

    pub fn default(reader: Box<dyn FormatReader>) -> Result<Self, SyphonError> {
        let (track_i, stream_spec) = reader
            .format_data()
            .tracks
            .iter()
            .enumerate()
            .find(|(_, t)| t.codec_key.is_some() && t.decoded_spec.sample_format.is_some())
            .map(|(i, spec)| (i, spec.try_build()))
            .ok_or(SyphonError::Empty)?;

        Ok(Self {
            reader,
            stream_spec: stream_spec?,
            track_i,
        })
    }
}

impl EncodedStreamReader for TrackReader {
    fn stream_spec(&self) -> &EncodedStreamSpec {
        &self.stream_spec
    }
}

impl Read for TrackReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let result = self.reader.read(buf)?;
            if result.track_i == self.track_i {
                return Ok(result.n_bytes);
            }
        }
    }
}

impl Seek for TrackReader {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        Ok(self.reader.seek(offset)?)
    }
}

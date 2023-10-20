use crate::{
    io::{EncodedStreamReader, EncodedStreamSpec, FormatReader},
    SyphonError,
};
use std::io::{self, Read, Seek, SeekFrom};

pub struct TrackReader<R: FormatReader> {
    reader: R,
    stream_spec: EncodedStreamSpec,
    track_i: usize,
}

impl<R: FormatReader> TrackReader<R> {
    pub fn new(reader: R, track_i: usize) -> Result<Self, SyphonError> {
        let stream_spec = reader
            .tracks()
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

    pub fn default(reader: R) -> Result<Self, SyphonError> {
        let (track_i, stream_spec) = reader
            .tracks()
            .iter()
            .enumerate()
            .find(|(_, spec)| spec.codec_key.is_some() && spec.decoded_spec.sample_format.is_some())
            .map(|(i, spec)| (i, spec.try_build()))
            .ok_or(SyphonError::Empty)?;

        Ok(Self {
            reader,
            stream_spec: stream_spec?,
            track_i,
        })
    }
}

impl<R: FormatReader> EncodedStreamReader for TrackReader<R> {
    fn stream_spec(&self) -> &EncodedStreamSpec {
        &self.stream_spec
    }
}

impl<R: FormatReader> Read for TrackReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let result = self.reader.read(buf)?;
            if result.track_i == self.track_i {
                return Ok(result.n);
            }
        }
    }
}

impl<R: FormatReader> Seek for TrackReader<R> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        Ok(self.reader.seek(offset)?)
    }
}

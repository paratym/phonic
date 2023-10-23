use crate::{
    io::{EncodedStream, EncodedStreamSpec, Format, FormatReader, FormatWriter},
    SyphonError,
};
use std::io::{self, Read, Seek, SeekFrom, Write};

pub struct Track<T> {
    inner: T,
    spec: EncodedStreamSpec,
    track_i: usize,
}

impl<T> Track<T> {
    pub fn default(inner: T) -> Result<Self, SyphonError>
    where
        T: Format,
    {
        let (track_i, spec) = inner
            .format_data()
            .tracks
            .iter()
            .enumerate()
            .find(|(_, spec)| spec.codec_key.is_some() && spec.decoded_spec.sample_format.is_some())
            .map(|(i, spec)| spec.build().map(|spec| (i, spec)))
            .ok_or(SyphonError::NotFound)??;

        Ok(Self {
            inner,
            spec,
            track_i,
        })
    }
}

impl<T: FormatReader> Read for Track<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let result = self.inner.read(buf)?;
            if result.track == self.track_i {
                return Ok(result.n);
            }
        }
    }
}

impl<T: FormatWriter> Write for Track<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(self.inner.write(self.track_i, buf)?)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(self.inner.flush()?)
    }
}

impl<T: Seek> Seek for Track<T> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        Ok(self.inner.seek(offset)?)
    }
}

impl<T> EncodedStream for Track<T> {
    fn spec(&self) -> &EncodedStreamSpec {
        &self.spec
    }
}

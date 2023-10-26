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
    pub fn from_format(inner: T, track_i: usize) -> Result<Self, SyphonError>
    where
        T: Format,
    {
        let spec = inner
            .format_data()
            .tracks
            .get(track_i)
            .ok_or(SyphonError::NotFound)?
            .build()?;

        Ok(Self {
            inner,
            spec,
            track_i,
        })
    }

    pub fn default_from_format(inner: T) -> Result<Self, SyphonError>
    where
        T: Format,
    {
        let default_i = inner
            .format_data()
            .tracks
            .iter()
            .position(|spec| spec.codec_key.is_some())
            .ok_or(SyphonError::NotFound)?;

        Self::from_format(inner, default_i)
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

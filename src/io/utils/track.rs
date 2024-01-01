use std::ops::{Deref, DerefMut};

use crate::{
    io::{Format, FormatReader, FormatWriter, Stream, StreamReader, StreamSpec, StreamWriter},
    SyphonError,
};

pub struct Track<F: Format> {
    inner: F,
    track_i: usize,
    spec: StreamSpec,
}

impl<F: Format> Track<F> {
    pub fn new(inner: F, track_i: usize) -> Result<Self, SyphonError> {
        let spec = *inner
            .format_data()
            .tracks
            .get(track_i)
            .ok_or(SyphonError::NotFound)?;

        Ok(Self {
            inner,
            track_i,
            spec,
        })
    }
}

impl<F: Format> Stream for Track<F> {
    fn spec(&self) -> &StreamSpec {
        &self.spec
    }
}

impl<T> StreamReader for Track<T>
where
    T: DerefMut,
    T::Target: FormatReader,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SyphonError> {
        loop {
            let result = self.inner.read(buf)?;
            if result.track == self.track_i {
                return Ok(result.n);
            }
        }
    }
}

impl<T> StreamWriter for Track<T>
where
    T: DerefMut,
    T::Target: FormatWriter,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, SyphonError> {
        Ok(self.inner.write(self.track_i, buf)?)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        Ok(self.inner.flush()?)
    }
}

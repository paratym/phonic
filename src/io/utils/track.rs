use crate::io::{FormatReader, FormatWriter, Stream, StreamSpec};
use std::io::{self, Read, Seek, SeekFrom, Write};

pub struct Track<T> {
    inner: T,
    track_i: usize,
    spec: StreamSpec,
}

impl<T> Track<T> {
    pub fn new(inner: T, track_i: usize, spec: StreamSpec) -> Self {
        Self {
            inner,
            spec,
            track_i,
        }
    }
}

impl<T> Stream for Track<T> {
    fn spec(&self) -> &StreamSpec {
        &self.spec
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

use phonic_signal::PhonicError;

use crate::{
    FiniteStream, Format, FormatReader, FormatSeeker, FormatTag, FormatWriter, IndexedStream,
    Stream, StreamReader, StreamSeeker, StreamSpec, StreamWriter,
};

pub struct StreamSelector<F> {
    inner: F,
    stream: usize,
}

impl<F> StreamSelector<F> {
    pub fn new(inner: F, stream: usize) -> Result<Self, PhonicError>
    where
        F: Format,
    {
        inner.streams().get(stream).ok_or(PhonicError::NotFound)?;
        Ok(Self { inner, stream })
    }
}

impl<F: Format> Stream for StreamSelector<F> {
    type Tag = <F::Tag as FormatTag>::Codec;

    fn stream_spec(&self) -> &StreamSpec<Self::Tag> {
        &self.inner.streams()[self.stream]
    }
}

impl<F: Format> IndexedStream for StreamSelector<F> {
    fn pos(&self) -> u64 {
        todo!()
    }
}

impl<F: Format> FiniteStream for StreamSelector<F> {
    fn len(&self) -> u64 {
        todo!()
    }
}

impl<T: FormatReader> StreamReader for StreamSelector<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, PhonicError> {
        loop {
            let (i, n) = self.inner.read(buf)?;
            if i == self.stream {
                return Ok(n);
            }
        }
    }
}

impl<T: FormatWriter> StreamWriter for StreamSelector<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, PhonicError> {
        self.inner.write(self.stream, buf)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.inner.flush()
    }
}

impl<T: FormatSeeker> StreamSeeker for StreamSelector<T> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        todo!()
    }
}

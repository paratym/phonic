use crate::{
    Format, FormatChunk, FormatObserver, FormatOffset, FormatPosition, FormatReader, FormatSeeker,
    FormatTag, FormatWriter, Stream, StreamObserver, StreamReader, StreamSeeker, StreamSpec,
    StreamWriter,
};
use phonic_core::PhonicError;

pub struct StreamSelector<F: Format> {
    inner: F,
    stream_i: usize,
}

impl<F: Format> StreamSelector<F> {
    pub fn new(inner: F, stream_i: usize) -> Result<Self, PhonicError> {
        if inner.data().streams.len() <= stream_i {
            return Err(PhonicError::NotFound);
        }

        Ok(Self { inner, stream_i })
    }
}

impl<F: Format> Stream for StreamSelector<F> {
    type Tag = <F::Tag as FormatTag>::Codec;

    fn spec(&self) -> &StreamSpec<Self::Tag> {
        &self.inner.data().streams[self.stream_i]
    }
}

impl<T: FormatObserver> StreamObserver for StreamSelector<T> {
    fn position(&self) -> Result<u64, PhonicError> {
        match self.inner.position()? {
            FormatPosition { stream_i, .. } if stream_i < self.stream_i => Ok(0),
            FormatPosition { stream_i, .. } if stream_i > self.stream_i => {
                self.spec().n_bytes().ok_or(PhonicError::OutOfBounds)
            }
            FormatPosition { byte_i, .. } => Ok(byte_i),
        }
    }
}

impl<T: FormatReader> StreamReader for StreamSelector<T> {
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, PhonicError> {
        loop {
            match self.inner.read(buffer)? {
                FormatChunk::Stream { stream_i, buf } if stream_i == self.stream_i => {
                    return Ok(buf.len());
                }
                _ => {}
            }
        }
    }
}

impl<T: FormatWriter> StreamWriter for StreamSelector<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, PhonicError> {
        let chunk = FormatChunk::Stream {
            stream_i: self.stream_i,
            buf,
        };

        self.inner.write(chunk)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        Ok(self.inner.flush()?)
    }
}

impl<T: FormatSeeker + FormatObserver> StreamSeeker for StreamSelector<T> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        let pos = self.inner.position()?;
        self.inner.seek(FormatOffset {
            stream_offset: pos.stream_i as isize - self.stream_i as isize,
            byte_offset: offset,
        })
    }
}

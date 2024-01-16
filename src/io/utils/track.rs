use crate::{
    io::{
        formats::KnownFormat, Format, FormatChunk, FormatReader, FormatWriter, Stream, StreamSpec,
        TrackChunk,
    },
    SyphonError,
};
use std::{
    io::{self, Read, Write},
    ops::DerefMut,
};

pub struct Track<F: Format> {
    inner: F,
    track_i: usize,
}

impl<F: Format> Track<F> {
    pub fn new(inner: F, track_i: usize) -> Result<Self, SyphonError> {
        if inner.data().tracks.len() <= track_i {
            return Err(SyphonError::NotFound);
        }

        Ok(Self { inner, track_i })
    }
}

impl<F: Format> Stream for Track<F> {
    type Codec = <F::Format as KnownFormat>::Codec;

    fn codec(&self) -> Option<&Self::Codec> {
        self.inner.data().tracks[self.track_i].0.as_ref()
    }

    fn spec(&self) -> &StreamSpec {
        &self.inner.data().tracks[self.track_i].1
    }
}

impl<T> Read for Track<T>
where
    T: DerefMut,
    T::Target: FormatReader,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        loop {
            match self.inner.read(buf)? {
                FormatChunk::Track(TrackChunk { i, buf }) if i == self.track_i => {
                    return Ok(buf.len());
                }
                _ => continue,
            }
        }
    }
}

impl<T> Write for Track<T>
where
    T: DerefMut,
    T::Target: FormatWriter,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let chunk = TrackChunk {
            i: self.track_i,
            buf,
        };

        self.inner.write_track_chunk(chunk).map_err(Into::into)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        Ok(self.inner.flush()?)
    }
}

use crate::{
    io::{
        Format, FormatChunk, FormatData, FormatReader, FormatWriter, Stream, StreamSpec, TrackChunk,
    },
    SyphonError,
};
use std::io::{Read, Write};

pub struct SingleStreamFormat<T> {
    inner: T,
    data: FormatData,
    spec: StreamSpec,
}

impl<T> SingleStreamFormat<T> {
    pub fn new(inner: T, data: FormatData) -> Result<Self, SyphonError> {
        if data.tracks.len() != 1 {
            return Err(SyphonError::InvalidData);
        }

        let spec = data.tracks[0];
        Ok(Self { inner, data, spec })
    }
}

impl<T> Stream for SingleStreamFormat<T> {
    fn spec(&self) -> &StreamSpec {
        &self.spec
    }
}

impl<T: Read> Read for SingleStreamFormat<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Write> Write for SingleStreamFormat<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<T> Format for SingleStreamFormat<T> {
    fn data(&self) -> &FormatData {
        &self.data
    }
}

impl<T> FormatReader for SingleStreamFormat<T>
where
    Self: Read,
{
    fn read<'a>(&mut self, buf: &'a mut [u8]) -> Result<FormatChunk<'a>, SyphonError> {
        Read::read(self, buf)
            .map(|n| {
                FormatChunk::Track(TrackChunk {
                    i: 0,
                    buf: &buf[..n],
                })
            })
            .map_err(Into::into)
    }
}

impl<T> FormatWriter for SingleStreamFormat<T>
where
    Self: Write,
{
    fn write_track_chunk(&mut self, chunk: TrackChunk) -> Result<usize, SyphonError> {
        if chunk.i != 0 {
            return Err(SyphonError::Unsupported);
        }

        Write::write(self, &chunk.buf).map_err(Into::into)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        Write::flush(self).map_err(Into::into)
    }
}

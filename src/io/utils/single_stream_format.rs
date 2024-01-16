use crate::{
    io::{
        Format, FormatChunk, FormatData, FormatReader, FormatWriter, Stream, StreamSpec, TrackChunk, formats::KnownFormat,
    },
    SyphonError,
};
use std::io::{Read, Write};

pub struct SingleStreamFormat<T, F: KnownFormat> {
    inner: T,
    format: Option<F>,
    data: FormatData<F>,
}

impl<T, F: KnownFormat> SingleStreamFormat<T, F> {
    pub fn new(inner: T, format: Option<F>, data: FormatData<F>) -> Result<Self, SyphonError> {
        if data.tracks.len() != 1 {
            return Err(SyphonError::InvalidData);
        }

        Ok(Self { inner, format, data })
    }
}

impl<T, F: KnownFormat> Stream for SingleStreamFormat<T, F> {
    type Codec = F::Codec;

    fn codec(&self) -> Option<&Self::Codec> {
        self.data.tracks[0].0.as_ref()
    }

    fn spec(&self) -> &StreamSpec {
        &self.data.tracks[0].1
    }
}

impl<T: Read, F: KnownFormat> Read for SingleStreamFormat<T, F> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Write, F: KnownFormat> Write for SingleStreamFormat<T, F> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<T, F: KnownFormat> Format for SingleStreamFormat<T, F> {
    type Format = F;

    fn format(&self) -> Option<Self::Format> {
        None
    }

    fn data(&self) -> &FormatData<Self::Format> {
        &self.data
    }
}

impl<T, F: KnownFormat> FormatReader for SingleStreamFormat<T, F>
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

impl<T, F: KnownFormat> FormatWriter for SingleStreamFormat<T, F>
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

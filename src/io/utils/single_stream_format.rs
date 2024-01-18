use crate::{
    io::{
        formats::FormatTag, Format, FormatChunk, FormatData, FormatReader, FormatWriter, Stream,
        StreamSpec,
    },
    SyphonError,
};
use std::io::{Read, Write};

pub struct SingleStreamFormat<T, F: FormatTag> {
    inner: T,
    data: FormatData<F>,
}

impl<T, F: FormatTag> SingleStreamFormat<T, F> {
    pub fn new(inner: T, data: FormatData<F>) -> Result<Self, SyphonError> {
        if data.streams.len() != 1 {
            return Err(SyphonError::InvalidData);
        }

        Ok(Self { inner, data })
    }
}

impl<T, F: FormatTag> Stream for SingleStreamFormat<T, F> {
    type Tag = F::Codec;

    fn spec(&self) -> &StreamSpec<Self::Tag> {
        &self.data.streams[0]
    }
}

impl<T: Read, F: FormatTag> Read for SingleStreamFormat<T, F> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Write, F: FormatTag> Write for SingleStreamFormat<T, F> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<T, F: FormatTag> Format for SingleStreamFormat<T, F> {
    type Tag = F;

    fn data(&self) -> &FormatData<Self::Tag> {
        &self.data
    }
}

impl<T, F: FormatTag> FormatReader for SingleStreamFormat<T, F>
where
    Self: Read,
{
    fn read<'a>(&mut self, buf: &'a mut [u8]) -> Result<FormatChunk<'a>, SyphonError> {
        Read::read(self, buf)
            .map(|n| FormatChunk::Stream {
                stream_i: 0,
                buf: &buf[..n],
            })
            .map_err(Into::into)
    }
}

impl<T, F: FormatTag> FormatWriter for SingleStreamFormat<T, F>
where
    Self: Write,
{
    fn write(&mut self, chunk: FormatChunk) -> Result<(), SyphonError> {
        match chunk {
            FormatChunk::Stream { stream_i, buf } if stream_i == 0 => {
                Write::write_all(self, buf).map_err(Into::into)
            }
            _ => Err(SyphonError::Unsupported),
        }
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        Write::flush(self).map_err(Into::into)
    }
}

use crate::{
    io::{
        Format, FormatChunk, FormatData, FormatReader, FormatWriter, Stream, StreamSpec,
        SyphonCodec, SyphonFormat, TrackChunk,
    },
    SyphonError,
};
use std::io::{self, Read, Write};

pub struct SingleStreamFormat<T> {
    inner: T,
    format: Option<SyphonFormat>,
    codec: Option<SyphonCodec>,
    data: FormatData,
}

impl<T> SingleStreamFormat<T> {
    pub fn new(inner: T, data: FormatData) -> Self {
        Self {
            inner,
            format: None,
            codec: None,
            data,
        }
    }

    pub fn with_format(mut self, format: SyphonFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn with_codec(mut self, codec: SyphonCodec) -> Self {
        self.codec = Some(codec);
        self
    }
}

impl<T> Stream for SingleStreamFormat<T> {
    fn spec(&self) -> &StreamSpec {
        &self.data.tracks[0]
    }
}

impl<T: Read> Read for SingleStreamFormat<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        self.inner.read(buf)
    }
}

impl<T: Write> Write for SingleStreamFormat<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
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

        Write::write(self, chunk.buf).map_err(Into::into)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        Write::flush(self).map_err(Into::into)
    }
}

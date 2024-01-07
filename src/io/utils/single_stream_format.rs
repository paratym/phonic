use crate::{
    io::{
        Format, FormatChunk, FormatData, FormatReader, FormatWriter, Stream, StreamSpecBuilder,
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

impl<T> Format for SingleStreamFormat<T> {
    fn data(&self) -> &FormatData {
        &self.data
    }
}

impl<T: Read> FormatReader for SingleStreamFormat<T> {
    fn read<'a>(&mut self, buf: &'a mut [u8]) -> Result<FormatChunk<'a>, SyphonError> {

        self.inner.read(buf)
            .map(|n| {
                FormatChunk::Track(TrackChunk {
                    i: 0,
                    buf: &buf[..n],
                })
            })
            .map_err(Into::into)
    }
}

impl<T: Write> FormatWriter for SingleStreamFormat<T> {
    fn write_track_chunk(&mut self, chunk: TrackChunk) -> Result<usize, SyphonError> {
        if chunk.i != 0 {
            return Err(SyphonError::Unsupported);
        }

        self.inner.write(&chunk.buf).map_err(Into::into)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.inner.flush().map_err(Into::into)
    }
}

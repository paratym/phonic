use crate::{WaveFormatTag, WaveHeader, WaveSupportedCodec};
use phonic_core::PhonicError;
use phonic_io_core::{
    Format, FormatChunk, FormatData, FormatObserver, FormatReader, FormatSeeker, FormatTag,
    FormatWriter, Stream, StreamReader, StreamSpec, StreamWriter,
};
use std::{
    io::{Read, Seek, Write},
    usize,
};

pub struct WaveFormat<T, F: FormatTag = WaveFormatTag> {
    inner: T,
    data: FormatData<F>,
    within_header: bool,
}

impl<T, F: FormatTag> WaveFormat<T, F> {
    pub fn new(inner: T) -> Result<Self, PhonicError>
    where
        WaveFormatTag: TryInto<F>,
    {
        let mut data = FormatData::new().with_stream(StreamSpec::new());
        data.format = WaveFormatTag.try_into().ok();

        Ok(Self {
            inner,
            data,
            within_header: true,
        })
    }
}

impl<T, F: FormatTag> Format for WaveFormat<T, F> {
    type Tag = F;

    fn data(&self) -> &FormatData<Self::Tag> {
        &self.data
    }
}

impl<T, F: FormatTag> FormatObserver for WaveFormat<T, F> {
    fn position(&self) -> Result<phonic_io_core::FormatPosition, PhonicError> {
        todo!()
    }
}

impl<T: Read, F: FormatTag> FormatReader for WaveFormat<T, F>
where
    WaveFormatTag: TryInto<F>,
    WaveSupportedCodec: TryInto<F::Codec>,
{
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> Result<FormatChunk<'a, Self::Tag>, PhonicError> {
        if self.within_header {
            let header = WaveHeader::read(&mut self.inner)?;
            self.data.merge(&header.into())?;
            self.within_header = false;
            return Ok(FormatChunk::Data { data: &self.data });
        }

        let n = StreamReader::read(self, buf)?;
        Ok(FormatChunk::Stream {
            stream_i: 0,
            buf: &buf[..n],
        })
    }
}

impl<T: Write, F: FormatTag> FormatWriter for WaveFormat<T, F> {
    fn write(&mut self, chunk: FormatChunk<Self::Tag>) -> Result<(), PhonicError> {
        match chunk {
            FormatChunk::Data { data } => {
                if !self.within_header {
                    todo!()
                }

                self.data.merge(data)?;
                let header = WaveHeader::try_from(&self.data)?;
                header.write(&mut self.inner)?;
            }
            FormatChunk::Stream { stream_i, buf } => {
                if stream_i != 0 {
                    return Err(PhonicError::NotFound);
                }

                StreamWriter::write_exact(self, buf)?;
            }
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.inner.flush().map_err(Into::into)
    }
}

impl<T: Seek, F: FormatTag> FormatSeeker for WaveFormat<T, F> {
    fn seek(&mut self, offset: phonic_io_core::FormatOffset) -> Result<(), PhonicError> {
        todo!()
    }
}

impl<T, F: FormatTag> Stream for WaveFormat<T, F> {
    type Tag = F::Codec;

    fn spec(&self) -> &StreamSpec<Self::Tag> {
        &self.data.streams[0]
    }
}

impl<T: Read, F: FormatTag> StreamReader for WaveFormat<T, F> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, PhonicError> {
        if self.within_header {
            return Err(PhonicError::NotReady);
        }

        let mut len = buf.len();
        let block_align = self.spec().block_align;
        if let Some(align) = block_align {
            len -= len % align as usize;
        }

        let n = self.inner.read(&mut buf[..len])?;
        if block_align.is_some_and(|align| n % align as usize != 0) {
            todo!()
        }

        Ok(n)
    }
}

impl<T: Write, F: FormatTag> StreamWriter for WaveFormat<T, F> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, PhonicError> {
        if self.within_header {
            return Err(PhonicError::NotReady);
        }

        let mut len = buf.len();
        let block_align = self.spec().block_align;
        if let Some(align) = block_align {
            len -= len % align as usize
        }

        let n = self.inner.write(&buf[..len])?;
        if block_align.is_some_and(|align| n % align as usize != 0) {
            todo!()
        }

        Ok(n)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        FormatWriter::flush(self)
    }
}

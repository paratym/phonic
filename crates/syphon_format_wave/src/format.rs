use crate::{WaveFormatTag, WaveHeader, WaveSupportedCodec};
use std::io::{Read, Seek, Write};
use syphon_core::SyphonError;
use syphon_io_core::{
    Format, FormatChunk, FormatData, FormatObserver, FormatReader, FormatSeeker, FormatTag,
    FormatWriter,
};

pub struct WaveFormat<T, F: FormatTag = WaveFormatTag> {
    inner: T,
    i: usize,
    data: FormatData<F>,
}

impl<T, F: FormatTag> WaveFormat<T, F> {
    pub fn new(inner: T) -> Result<Self, SyphonError>
    where
        WaveFormatTag: TryInto<F>,
    {
        let mut data = FormatData::new();
        data.format = WaveFormatTag.try_into().ok();
        Ok(Self { inner, i: 0, data })
    }

    fn trim_buf_len(&self, mut len: usize) -> usize {
        let stream_spec = self.data.streams.get(0);
        // if let Some(n_bytes) = stream_spec.and_then(StreamSpec::n_bytes) {
        //     len = len.min(n_bytes as usize);
        // }

        if let Some(block_align) = stream_spec.and_then(|spec| spec.block_align) {
            len = len - len % block_align as usize;
        }

        len
    }
}

impl<T, F: FormatTag> Format for WaveFormat<T, F> {
    type Tag = F;

    fn data(&self) -> &FormatData<Self::Tag> {
        &self.data
    }
}

impl<T, F: FormatTag> FormatObserver for WaveFormat<T, F> {
    fn position(&self) -> Result<syphon_io_core::FormatPosition, SyphonError> {
        todo!()
    }
}

impl<T: Read, F: FormatTag> FormatReader for WaveFormat<T, F>
where
    WaveFormatTag: TryInto<F>,
    WaveSupportedCodec: TryInto<F::Codec>,
{
    fn read_data(&mut self) -> Result<(), SyphonError> {
        if self.i > 0 {
            return Ok(());
        }

        self.data
            .merge(&WaveHeader::read(&mut self.inner)?.into())?;
        return Ok(());
    }

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Result<FormatChunk<'a>, SyphonError> {
        if self.i == 0 {
            self.read_data()?;
        }

        let len = self.trim_buf_len(buf.len());
        let n = self.inner.read(&mut buf[..len])?;
        self.i += n;

        if self
            .data
            .streams
            .get(0)
            .and_then(|spec| spec.block_align)
            .is_some_and(|align| n % align as usize != 0)
        {
            return Err(SyphonError::SignalMismatch);
        }

        Ok(FormatChunk::Stream {
            stream_i: 0,
            buf: &buf[..n],
        })
    }
}

impl<T: Write, F: FormatTag> FormatWriter for WaveFormat<T, F> {
    fn write_data(&mut self, data: &FormatData<F>) -> Result<(), SyphonError> {
        self.data.merge(data)?;
        let header = WaveHeader::try_from(&self.data)?;
        header.write(&mut self.inner)?;
        self.i += header.byte_len() as usize;

        Ok(())
    }

    fn write(&mut self, chunk: FormatChunk) -> Result<(), SyphonError> {
        match chunk {
            FormatChunk::Stream { stream_i, buf } if self.i > 0 && stream_i == 0 => {
                if buf.len() != self.trim_buf_len(buf.len()) {
                    return Err(SyphonError::SignalMismatch);
                }

                self.inner.write_all(buf)?;
                self.i += buf.len();
            }
            _ => return Err(SyphonError::InvalidData),
        }

        Ok(())
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.inner.flush().map_err(Into::into)
    }
}

impl<T: Seek, F: FormatTag> FormatSeeker for WaveFormat<T, F> {
    fn seek(&mut self, offset: syphon_io_core::FormatOffset) -> Result<(), SyphonError> {
        todo!()
    }
}

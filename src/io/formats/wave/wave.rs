use crate::{
    io::{
        codecs::SyphonCodec,
        formats::{wave::WaveHeader, FormatIdentifiers, SyphonFormat},
        Format, FormatChunk, FormatData, FormatReader, FormatWriter, StreamSpec,
    },
    SyphonError,
};
use std::io::{Read, Write};

pub static WAVE_IDENTIFIERS: FormatIdentifiers = FormatIdentifiers {
    file_extensions: &["wav", "wave"],
    mime_types: &["audio/vnd.wave", "audio/x-wav", "audio/wav", "audio/wave"],
    markers: &[b"RIFF", b"WAVE"],
};

pub fn fill_wave_format_data(data: &mut FormatData<SyphonFormat>) -> Result<(), SyphonError> {
    if data.format.get_or_insert(SyphonFormat::Wave) != &SyphonFormat::Wave
        || data.streams.len() > 1
    {
        return Err(SyphonError::InvalidData);
    }

    if data.streams.is_empty() {
        data.streams.push(StreamSpec::new());
    }

    let stream_spec = data.streams.first_mut().ok_or(SyphonError::InvalidData)?;
    if stream_spec.codec.get_or_insert(SyphonCodec::Pcm) != &SyphonCodec::Pcm {
        return Err(SyphonError::InvalidData);
    }

    stream_spec.fill()?;
    Ok(())
}

pub struct WaveFormat<T> {
    inner: T,
    i: usize,
    data: FormatData<SyphonFormat>,
}

impl<T> WaveFormat<T> {
    pub fn new(inner: T) -> Result<Self, SyphonError> {
        Ok(Self {
            inner,
            i: 0,
            data: FormatData::new().with_format(SyphonFormat::Wave),
        })
    }

    fn trim_buf_len(&self, mut len: usize) -> usize {
        let stream_spec = self.data.streams.get(0);
        if let Some(n_bytes) = stream_spec.and_then(StreamSpec::n_bytes) {
            len = len.min(n_bytes as usize);
        }

        if let Some(block_align) = stream_spec.and_then(|spec| spec.block_align) {
            len = len % block_align as usize;
        }

        len
    }
}

impl<T> Format for WaveFormat<T> {
    type Tag = SyphonFormat;

    fn data(&self) -> &FormatData<Self::Tag> {
        &self.data
    }
}

impl<T: Read> FormatReader for WaveFormat<T> {
    fn fill_data(&mut self) -> Result<(), SyphonError> {
        if self.i > 0 {
            return Ok(());
        }

        self.data
            .merge(&WaveHeader::read(&mut self.inner)?.into())?;
        return Ok(());
    }

    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> Result<FormatChunk<'a, Self::Tag>, SyphonError> {
        if self.i == 0 {
            self.fill_data()?;
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

impl<T: Write> FormatWriter for WaveFormat<T> {
    fn write(&mut self, chunk: FormatChunk<'_, Self::Tag>) -> Result<(), SyphonError> {
        match chunk {
            FormatChunk::Data(data) if self.i == 0 => {
                self.data.merge(data)?;
                let header = WaveHeader::try_from(&self.data)?;
                header.write(&mut self.inner)?;
                self.i += header.byte_len() as usize;
            }
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

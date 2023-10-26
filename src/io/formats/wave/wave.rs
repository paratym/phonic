use crate::{
    io::{
        formats::wave::WaveHeader, EncodedStream, EncodedStreamSpec, Format, FormatData,
        FormatIdentifiers, FormatReadResult, FormatReader, FormatWriter,
    },
    SyphonError,
};
use std::io::{Read, Seek, SeekFrom, Write};

pub static WAVE_IDENTIFIERS: FormatIdentifiers = FormatIdentifiers {
    file_extensions: &["wav", "wave"],
    mime_types: &["audio/vnd.wave", "audio/x-wav", "audio/wav", "audio/wave"],
    markers: &[b"RIFF", b"WAVE"],
};

pub fn fill_wave_data(data: &mut FormatData) -> Result<(), SyphonError> {
    Ok(())
}

pub struct Wave<T> {
    header: WaveHeader,
    inner: T,
    i: u64,
}

impl<T> Wave<T> {
    pub fn read(mut inner: T) -> std::io::Result<Self>
    where
        T: Read,
    {
        let header = WaveHeader::read(&mut inner)?;

        Ok(Self {
            header,
            inner,
            i: 0,
        })
    }

    pub fn write(mut inner: T, header: WaveHeader) -> std::io::Result<Self>
    where
        T: Write,
    {
        header.write(&mut inner)?;

        Ok(Self {
            header,
            inner,
            i: 0,
        })
    }

    pub fn header(&self) -> &WaveHeader {
        &self.header
    }

    pub fn into_format(self) -> WavFormat<T> {
        WavFormat::from(self)
    }

    pub fn into_stream(self) -> Result<WavEncodedStream<T>, SyphonError> {
        WavEncodedStream::try_from(self)
    }
}

impl<T: Read> Read for Wave<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut buf_len = buf
            .len()
            .min(self.header.data.byte_len as usize - self.i as usize);
        buf_len -= buf_len % self.header.fmt.block_align as usize;

        let n = self.inner.read(&mut buf[..buf_len])?;
        self.i += n as u64;

        if n % self.header.fmt.block_align as usize != 0 {
            todo!();
        }

        Ok(n)
    }
}

impl<T: Write> Write for Wave<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut buf_len = buf
            .len()
            .min(self.header.data.byte_len as usize - self.i as usize);
        buf_len -= buf_len % self.header.fmt.block_align as usize;

        let n = self.inner.write(&buf[..buf_len])?;
        self.i += n as u64;

        if n % self.header.fmt.block_align as usize != 0 {
            todo!();
        }

        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for Wave<T> {
    fn seek(&mut self, offset: SeekFrom) -> std::io::Result<u64> {
        todo!()

        // let new_pos = match offset {
        //     SeekFrom::Current(offset) if offset == 0 => {
        //         return Ok(self.media_pos());
        //     }
        //     SeekFrom::Current(offset) => self.source_pos as i64 + offset,
        //     SeekFrom::Start(offset) => self.media_bounds.0 as i64 + offset as i64,
        //     SeekFrom::End(offset) => self.media_bounds.1 as i64 + offset,
        // };

        // if new_pos < 0 {
        //     return Err(SyphonError::BadRequest);
        // }

        // let mut new_pos = new_pos as u64;
        // if new_pos < self.media_bounds.0 || new_pos >= self.media_bounds.1 {
        //     return Err(SyphonError::BadRequest);
        // }

        // let new_media_pos = new_pos - self.media_bounds.0;
        // let block_size = self.stream_spec_mut().block_size.unwrap_or(1);
        // new_pos -= new_media_pos % block_size as u64;

        // self.source_pos = self.source.seek(SeekFrom::Start(new_pos))?;
        // Ok(self.media_pos())
    }
}

pub struct WavFormat<T> {
    inner: Wave<T>,
    data: FormatData,
}

impl<T> From<Wave<T>> for WavFormat<T> {
    fn from(inner: Wave<T>) -> Self {
        let data = inner.header.into();
        Self { inner, data }
    }
}

impl<T: Read> Read for WavFormat<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Write> Write for WavFormat<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for WavFormat<T> {
    fn seek(&mut self, offset: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(offset)
    }
}

impl<T> Format for WavFormat<T> {
    fn format_data(&self) -> &FormatData {
        &self.data
    }
}

impl<T: Read> FormatReader for WavFormat<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError> {
        let n = self.inner.read(buf)?;
        Ok(FormatReadResult { track: 0, n })
    }
}

impl<T: Write> FormatWriter for WavFormat<T> {
    fn write(&mut self, track_i: usize, buf: &[u8]) -> Result<usize, SyphonError> {
        if track_i != 0 {
            return Err(SyphonError::Unsupported);
        }

        Ok(self.inner.write(buf)?)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        Ok(self.inner.flush()?)
    }
}

pub struct WavEncodedStream<T> {
    inner: Wave<T>,
    spec: EncodedStreamSpec,
}

impl<T> TryFrom<Wave<T>> for WavEncodedStream<T> {
    type Error = SyphonError;

    fn try_from(inner: Wave<T>) -> Result<Self, Self::Error> {
        let spec = Into::<FormatData>::into(inner.header)
            .tracks
            .first()
            .ok_or(SyphonError::NotFound)?
            .build()?;

        Ok(Self { inner, spec })
    }
}

impl<T> EncodedStream for WavEncodedStream<T> {
    fn spec(&self) -> &EncodedStreamSpec {
        &self.spec
    }
}

impl<T: Read> Read for WavEncodedStream<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Write> Write for WavEncodedStream<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for WavEncodedStream<T> {
    fn seek(&mut self, offset: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(offset)
    }
}

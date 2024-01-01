use crate::{
    io::{
        formats::{wave::WaveHeader, FormatIdentifiers, FormatWrapper},
        FormatDataBuilder, StreamSpec, SyphonCodec,
    },
    SyphonError,
};
use std::io::{self, Read, Write};

pub static WAVE_IDENTIFIERS: FormatIdentifiers = FormatIdentifiers {
    file_extensions: &["wav", "wave"],
    mime_types: &["audio/vnd.wave", "audio/x-wav", "audio/wav", "audio/wave"],
    markers: &[b"RIFF", b"WAVE"],
};

pub struct WaveFormat<T> {
    header: WaveHeader,
    inner: T,
    i: usize,
}

impl<T> WaveFormat<T> {
    pub fn read(mut inner: T) -> Result<Self, SyphonError>
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

    pub fn write(mut inner: T, header: WaveHeader) -> Result<Self, SyphonError>
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

    pub fn into_format(self) -> Result<FormatWrapper<Self>, SyphonError> {
        let data = FormatDataBuilder::from(self.header).build()?;
        Ok(FormatWrapper::new(self, data))
    }
}

impl<T: Read> Read for WaveFormat<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut buf_len = buf.len();
        buf_len = buf_len.min(self.header.data.byte_len as usize - self.i);
        buf_len -= buf_len % self.header.fmt.block_align as usize;

        let n = self.inner.read(&mut buf[..buf_len])?;
        self.i += n;

        if n % self.header.fmt.block_align as usize != 0 {
            todo!();
        }

        Ok(n)
    }
}

impl<T: Write> Write for WaveFormat<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut buf_len = buf.len();
        buf_len = buf_len.min(self.header.data.byte_len as usize - self.i);
        buf_len -= buf_len % self.header.fmt.block_align as usize;

        let n = self.inner.write(&buf[..buf_len])?;
        self.i += n;

        if n % self.header.fmt.block_align as usize != 0 {
            todo!();
        }

        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

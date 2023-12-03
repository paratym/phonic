use crate::{
    io::{
        formats::{wave::WaveHeader, FormatIdentifiers},
        Format, FormatData, FormatDataBuilder, FormatReadResult, FormatReader, FormatWriter,
        Stream, StreamSpec, SyphonCodec,
    },
    SampleType, SyphonError,
};
use std::io::{self, Read, Seek, SeekFrom, Write};

pub static WAVE_IDENTIFIERS: FormatIdentifiers = FormatIdentifiers {
    file_extensions: &["wav", "wave"],
    mime_types: &["audio/vnd.wave", "audio/x-wav", "audio/wav", "audio/wave"],
    markers: &[b"RIFF", b"WAVE"],
};

pub fn fill_wave_data(data: &mut FormatDataBuilder) -> Result<(), SyphonError> {
    if data.tracks.len() != 1 {
        return Err(SyphonError::Unsupported);
    }

    let track = data.tracks.first_mut().unwrap();

    if track.codec.is_none() {
        track.codec = match track.decoded_spec.sample_type {
            Some(SampleType::U8)
            | Some(SampleType::I16)
            | Some(SampleType::I32)
            | Some(SampleType::F32)
            | Some(SampleType::F64) => Some(SyphonCodec::Pcm),
            Some(_) => return Err(SyphonError::Unsupported),
            None => None,
        }
    }

    if track.codec.is_some_and(|codec| codec != SyphonCodec::Pcm) {
        return Err(SyphonError::Unsupported);
    }

    Ok(())
}

pub struct Wave<T> {
    header: WaveHeader,
    inner: T,
    i: usize,
}

impl<T> Wave<T> {
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

    pub fn into_format(self) -> Result<WaveFormat<T>, SyphonError> {
        WaveFormat::try_from(self)
    }

    pub fn into_stream(self) -> Result<WaveStream<T>, SyphonError> {
        WaveStream::try_from(self)
    }
}

impl<T: Read> Read for Wave<T> {
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

impl<T: Write> Write for Wave<T> {
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

impl<T: Seek> Seek for Wave<T> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        todo!()
    }
}

pub struct WaveFormat<T> {
    inner: Wave<T>,
    data: FormatData,
}

impl<T> TryFrom<Wave<T>> for WaveFormat<T> {
    type Error = SyphonError;

    fn try_from(inner: Wave<T>) -> Result<Self, Self::Error> {
        let data = FormatDataBuilder::from(inner.header).build()?;
        Ok(Self { inner, data })
    }
}

impl<T: Read> Read for WaveFormat<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Write> Write for WaveFormat<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for WaveFormat<T> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        self.inner.seek(offset)
    }
}

impl<T> Format for WaveFormat<T> {
    fn format_data(&self) -> &FormatData {
        &self.data
    }
}

impl<T: Read> FormatReader for WaveFormat<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError> {
        let n = self.inner.read(buf)?;
        Ok(FormatReadResult { track: 0, n })
    }
}

impl<T: Write> FormatWriter for WaveFormat<T> {
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

pub struct WaveStream<T> {
    inner: Wave<T>,
    spec: StreamSpec,
}

impl<T> TryFrom<Wave<T>> for WaveStream<T> {
    type Error = SyphonError;

    fn try_from(inner: Wave<T>) -> Result<Self, Self::Error> {
        let spec = FormatDataBuilder::from(inner.header)
            .tracks
            .first()
            .ok_or(SyphonError::NotFound)?
            .build()?;

        Ok(Self { inner, spec })
    }
}

impl<T> Stream for WaveStream<T> {
    fn spec(&self) -> &StreamSpec {
        &self.spec
    }
}

impl<T: Read> Read for WaveStream<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Write> Write for WaveStream<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for WaveStream<T> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        self.inner.seek(offset)
    }
}

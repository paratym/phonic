use crate::{WaveFormatTag, WaveHeader};
use phonic_io_core::{
    FiniteFormat, FiniteStream, Format, FormatConstructor, FormatReader, FormatSeeker, FormatTag,
    FormatWriter, IndexedFormat, IndexedStream, Stream, StreamReader, StreamSeeker, StreamSpec,
    StreamWriter,
};
use phonic_signal::{PhonicError, PhonicResult};
use std::io::{Read, Seek, Write};

pub struct WaveFormat<T, F: FormatTag = WaveFormatTag> {
    inner: T,
    spec: StreamSpec<F::Codec>,
    pos: u64,
}

impl<T, F: FormatTag> FormatConstructor<T, F> for WaveFormat<T, F>
where
    StreamSpec<F::Codec>: TryInto<WaveHeader, Error = PhonicError>,
    WaveHeader: TryInto<StreamSpec<F::Codec>, Error = PhonicError>,
{
    fn read_index(mut inner: T) -> PhonicResult<Self>
    where
        Self: Format<Tag = F>,
        T: Read,
    {
        let header = WaveHeader::read(&mut inner)?;
        let spec = header.try_into()?;

        Ok(Self {
            inner,
            spec,
            pos: 0,
        })
    }

    fn write_index<I>(mut inner: T, index: I) -> PhonicResult<Self>
    where
        Self: Format<Tag = F>,
        T: Write,
        I: IntoIterator<Item = StreamSpec<F::Codec>>,
    {
        let mut streams = index.into_iter();
        let spec = streams.next().ok_or(PhonicError::MissingData)?;
        if streams.next().is_some() {
            return Err(PhonicError::Unsupported);
        }

        let header: WaveHeader = spec.try_into()?;
        header.write(&mut inner)?;

        Ok(Self {
            inner,
            spec,
            pos: 0,
        })
    }
}

impl<T, F: FormatTag> WaveFormat<T, F> {
    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T, F> Format for WaveFormat<T, F>
where
    F: FormatTag,
    WaveFormatTag: Into<F>,
{
    type Tag = F;

    fn format(&self) -> Self::Tag {
        WaveFormatTag.into()
    }

    fn streams(&self) -> &[StreamSpec<<Self::Tag as FormatTag>::Codec>] {
        std::slice::from_ref(&self.spec)
    }

    fn current_stream(&self) -> usize {
        0
    }

    fn primary_stream(&self) -> Option<usize> {
        Some(0)
    }
}

impl<T, F> IndexedFormat for WaveFormat<T, F>
where
    F: FormatTag,
    Self: Format<Tag = F> + IndexedStream<Tag = F::Codec>,
{
    fn pos(&self) -> u64 {
        IndexedStream::pos(self)
    }

    fn stream_pos(&self, stream: usize) -> u64 {
        match stream {
            0 => IndexedStream::pos(self),
            _ => 0,
        }
    }
}

impl<T, F> FiniteFormat for WaveFormat<T, F>
where
    F: FormatTag,
    Self: Format<Tag = F> + FiniteStream<Tag = F::Codec>,
{
    fn len(&self) -> u64 {
        FiniteStream::len(self)
    }

    fn stream_len(&self, stream: usize) -> u64 {
        match stream {
            0 => FiniteStream::len(self),
            _ => 0,
        }
    }
}

impl<T, F> FormatReader for WaveFormat<T, F>
where
    T: Read,
    F: FormatTag,
    Self: Format<Tag = F> + StreamReader<Tag = F::Codec>,
{
    fn read(&mut self, buf: &mut [u8]) -> PhonicResult<(usize, usize)> {
        let n = StreamReader::read(self, buf)?;
        Ok((0, n))
    }
}

impl<T, F> FormatWriter for WaveFormat<T, F>
where
    T: Write,
    F: FormatTag,
    Self: Format<Tag = F> + StreamWriter<Tag = F::Codec>,
{
    fn write(&mut self, stream: usize, buf: &[u8]) -> PhonicResult<usize> {
        match stream {
            0 => StreamWriter::write(self, buf),
            _ => Err(PhonicError::NotFound),
        }
    }

    fn flush(&mut self) -> PhonicResult<()> {
        StreamWriter::flush(self)
    }

    fn finalize(&mut self) -> PhonicResult<()> {
        todo!()
    }
}

impl<T, F> FormatSeeker for WaveFormat<T, F>
where
    T: Seek,
    F: FormatTag,
    Self: Format<Tag = F> + StreamSeeker<Tag = F::Codec>,
{
    fn seek(&mut self, stream: usize, offset: i64) -> PhonicResult<()> {
        todo!()
    }
}

impl<T, F: FormatTag> Stream for WaveFormat<T, F> {
    type Tag = F::Codec;

    fn stream_spec(&self) -> &StreamSpec<Self::Tag> {
        &self.spec
    }
}

impl<T, F: FormatTag> IndexedStream for WaveFormat<T, F> {
    fn pos(&self) -> u64 {
        self.pos
    }
}

impl<T, F: FormatTag> FiniteStream for WaveFormat<T, F> {
    fn len(&self) -> u64 {
        todo!()
    }
}

impl<T: Read, F: FormatTag> StreamReader for WaveFormat<T, F> {
    fn read(&mut self, buf: &mut [u8]) -> PhonicResult<usize> {
        let mut len = buf.len();
        len -= len % self.stream_spec().block_align;

        let mut n = 0;
        loop {
            match self.inner.read(&mut buf[n..len])? {
                0 if n == 0 => break,
                0 => return Err(PhonicError::InvalidState),
                n_read => n += n_read,
            }

            if n % self.spec.block_align == 0 {
                break;
            }
        }

        self.pos += n as u64;
        Ok(n)
    }
}

impl<T: Write, F: FormatTag> StreamWriter for WaveFormat<T, F> {
    fn write(&mut self, buf: &[u8]) -> PhonicResult<usize> {
        let mut len = buf.len();
        len -= len % self.stream_spec().block_align;

        let mut n = 0;
        loop {
            match self.inner.write(&buf[n..len])? {
                0 if n == 0 => break,
                0 => return Err(PhonicError::InvalidData),
                n_written => n += n_written,
            }

            if n % self.spec.block_align == 0 {
                break;
            }
        }

        self.pos += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush().map_err(Into::into)
    }
}

impl<T: Seek, F: FormatTag> StreamSeeker for WaveFormat<T, F> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        todo!()
    }
}

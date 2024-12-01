use crate::{utils::StreamSelector, CodecTag, StreamSpec};
use phonic_signal::{PhonicError, PhonicResult};
use std::{
    fmt::Debug,
    hash::Hash,
    io::{Read, Write},
    ops::{Deref, DerefMut, Neg},
};

pub trait FormatTag: Sized + Send + Sync + Debug + Copy + Eq + Hash {
    type Codec: CodecTag;
}

pub trait FormatConstructor<T, F: FormatTag>: Sized {
    fn read_index(inner: T) -> PhonicResult<Self>
    where
        Self: Format<Tag = F>,
        T: Read;

    fn write_index<I>(inner: T, index: I) -> PhonicResult<Self>
    where
        Self: Format<Tag = F>,
        T: Write,
        I: IntoIterator<Item = StreamSpec<F::Codec>>;
}

pub trait Format {
    type Tag: FormatTag;

    fn format(&self) -> Self::Tag;
    fn streams(&self) -> &[StreamSpec<<Self::Tag as FormatTag>::Codec>];
    fn current_stream(&self) -> usize;

    fn primary_stream(&self) -> Option<usize> {
        match self.streams() {
            [_] => Some(0),
            [..] => None,
        }
    }

    fn as_stream(&mut self, stream: usize) -> Option<StreamSelector<&mut Self>>
    where
        Self: Sized,
    {
        StreamSelector::new(self, stream)
    }

    fn into_stream(self, stream: usize) -> Option<StreamSelector<Self>>
    where
        Self: Sized,
    {
        StreamSelector::new(self, stream)
    }

    fn current_stream_spec(&self) -> &StreamSpec<<Self::Tag as FormatTag>::Codec> {
        let i = self.current_stream();
        &self.streams()[i]
    }

    fn as_current_stream(&mut self) -> StreamSelector<&mut Self>
    where
        Self: Sized,
    {
        let i = self.current_stream();
        self.as_stream(i).unwrap()
    }

    fn into_current_stream(self) -> StreamSelector<Self>
    where
        Self: Sized,
    {
        let i = self.current_stream();
        self.into_stream(i).unwrap()
    }

    fn primary_stream_spec(&self) -> Option<&StreamSpec<<Self::Tag as FormatTag>::Codec>> {
        self.primary_stream().and_then(|i| self.streams().get(i))
    }

    fn as_primary_stream(&mut self) -> PhonicResult<StreamSelector<&mut Self>>
    where
        Self: Sized,
    {
        let i = self.primary_stream().ok_or(PhonicError::Unsupported)?;
        self.as_stream(i).ok_or(PhonicError::NotFound)
    }

    fn into_primary_stream(self) -> PhonicResult<StreamSelector<Self>>
    where
        Self: Sized,
    {
        let i = self.primary_stream().ok_or(PhonicError::Unsupported)?;
        self.into_stream(i).ok_or(PhonicError::NotFound)
    }
}

pub trait IndexedFormat: Format {
    fn pos(&self) -> u64;
    fn stream_pos(&self, stream: usize) -> u64;
}

pub trait FiniteFormat: Format {
    fn len(&self) -> u64;
    fn stream_len(&self, stream: usize) -> u64;

    fn is_empty(&self) -> bool
    where
        Self: Sized + IndexedFormat,
    {
        self.pos() == self.len()
    }
}

pub trait FormatReader: Format {
    fn read(&mut self, buf: &mut [u8]) -> PhonicResult<(usize, usize)>;
}

pub trait FormatWriter: Format {
    fn write(&mut self, stream: usize, buf: &[u8]) -> PhonicResult<usize>;
    fn flush(&mut self) -> PhonicResult<()>;
    fn finalize(&mut self) -> PhonicResult<()>;
}

pub trait FormatSeeker: Format {
    fn seek(&mut self, stream: usize, offset: i64) -> PhonicResult<()>;

    fn set_pos(&mut self, stream: usize, pos: u64) -> Result<(), PhonicError>
    where
        Self: Sized + IndexedFormat,
    {
        let current_pos = self.stream_pos(stream);
        let offset = if pos >= current_pos {
            (pos - current_pos) as i64
        } else {
            ((current_pos - pos) as i64).neg()
        };

        self.seek(stream, offset)
    }
}

impl<T> Format for T
where
    T: Deref,
    T::Target: Format,
{
    type Tag = <T::Target as Format>::Tag;

    fn format(&self) -> Self::Tag {
        self.deref().format()
    }

    fn streams(&self) -> &[StreamSpec<<Self::Tag as FormatTag>::Codec>] {
        self.deref().streams()
    }

    fn current_stream(&self) -> usize {
        self.deref().current_stream()
    }

    fn primary_stream(&self) -> Option<usize> {
        self.deref().primary_stream()
    }
}

impl<T> IndexedFormat for T
where
    T: Deref,
    T::Target: IndexedFormat,
{
    fn pos(&self) -> u64 {
        self.deref().pos()
    }

    fn stream_pos(&self, stream: usize) -> u64 {
        self.deref().stream_pos(stream)
    }
}

impl<T> FiniteFormat for T
where
    T: Deref,
    T::Target: FiniteFormat,
{
    fn len(&self) -> u64 {
        self.deref().len()
    }

    fn stream_len(&self, stream: usize) -> u64 {
        self.deref().stream_len(stream)
    }
}

impl<T> FormatReader for T
where
    T: DerefMut,
    T::Target: FormatReader,
{
    fn read(&mut self, buf: &mut [u8]) -> PhonicResult<(usize, usize)> {
        self.deref_mut().read(buf)
    }
}

impl<T> FormatWriter for T
where
    T: DerefMut,
    T::Target: FormatWriter,
{
    fn write(&mut self, stream: usize, buf: &[u8]) -> PhonicResult<usize> {
        self.deref_mut().write(stream, buf)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.deref_mut().flush()
    }

    fn finalize(&mut self) -> PhonicResult<()> {
        self.deref_mut().finalize()
    }
}

impl<T> FormatSeeker for T
where
    T: DerefMut,
    T::Target: FormatSeeker,
{
    fn seek(&mut self, stream: usize, offset: i64) -> PhonicResult<()> {
        self.deref_mut().seek(stream, offset)
    }
}

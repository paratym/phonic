use crate::{utils::StreamSelector, CodecTag, StreamSpec};
use phonic_core::PhonicError;
use std::{
    fmt::Debug,
    hash::Hash,
    io::{Read, Write},
    ops::{Deref, DerefMut},
};

pub trait FormatTag: Sized + Send + Sync + Debug + Copy + Eq + Hash {
    type Codec: CodecTag;
}

pub trait FormatConstructor<T, F: FormatTag>: Sized {
    fn read_index(inner: T) -> Result<Self, PhonicError>
    where
        Self: Format<Tag = F>,
        T: Read;

    fn write_index<I>(inner: T, index: I) -> Result<Self, PhonicError>
    where
        Self: Format<Tag = F>,
        T: Write,
        I: IntoIterator<Item = StreamSpec<F::Codec>>;
}

pub trait Format {
    type Tag: FormatTag;

    fn format(&self) -> Self::Tag;
    fn streams(&self) -> &[StreamSpec<<Self::Tag as FormatTag>::Codec>];

    fn default_stream(&self) -> Option<usize> {
        match self.streams() {
            [] => None,
            [..] => Some(0),
        }
    }

    fn default_stream_spec(&self) -> Option<&StreamSpec<<Self::Tag as FormatTag>::Codec>> {
        self.default_stream().and_then(|i| self.streams().get(i))
    }

    fn as_stream(&mut self, i: usize) -> Result<StreamSelector<&mut Self>, PhonicError>
    where
        Self: Sized,
    {
        StreamSelector::new(self, i)
    }

    fn as_default_stream(&mut self) -> Result<StreamSelector<&mut Self>, PhonicError>
    where
        Self: Sized,
    {
        self.default_stream()
            .ok_or(PhonicError::NotFound)
            .and_then(|i| self.as_stream(i))
    }

    fn into_stream(self, i: usize) -> Result<StreamSelector<Self>, PhonicError>
    where
        Self: Sized,
    {
        StreamSelector::new(self, i)
    }

    fn into_default_stream(self) -> Result<StreamSelector<Self>, PhonicError>
    where
        Self: Sized,
    {
        self.default_stream()
            .ok_or(PhonicError::NotFound)
            .and_then(|i| self.into_stream(i))
    }
}

pub trait FormatReader: Format {
    fn read(&mut self, buf: &mut [u8]) -> Result<(usize, usize), PhonicError>;

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<u32, PhonicError> {
        todo!()
    }
}

pub trait FormatWriter: Format {
    fn write(&mut self, stream: usize, buf: &[u8]) -> Result<usize, PhonicError>;
    fn flush(&mut self) -> Result<(), PhonicError>;
    fn finalize(&mut self) -> Result<(), PhonicError>;

    fn write_exact(&mut self, stream: usize, buf: &[u8]) -> Result<(), PhonicError> {
        todo!()
    }

    fn flushed(mut self) -> Result<Self, PhonicError>
    where
        Self: Sized,
    {
        self.flush()?;
        Ok(self)
    }

    fn finalized(mut self) -> Result<Self, PhonicError>
    where
        Self: Sized,
    {
        self.finalize()?;
        Ok(self)
    }
}

pub trait FormatSeeker: Format {
    fn seek(&mut self, stream: usize, offset: i64) -> Result<(), PhonicError>;

    // fn set_position(&mut self, position: FormatPosition) -> Result<(), PhonicError>
    // where
    //     Self: Sized + FormatObserver,
    // {
    //     let current_pos = self.position()?;
    //     self.seek(FormatOffset {
    //         stream_offset: current_pos.stream_i as isize - position.stream_i as isize,
    //         byte_offset: current_pos.byte_i as i64 - position.byte_i as i64,
    //     })
    // }
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

    fn default_stream(&self) -> Option<usize> {
        self.deref().default_stream()
    }
}

impl<T> FormatReader for T
where
    T: DerefMut,
    T::Target: FormatReader,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<(usize, usize), PhonicError> {
        self.deref_mut().read(buf)
    }
}

impl<T> FormatWriter for T
where
    T: DerefMut,
    T::Target: FormatWriter,
{
    fn write(&mut self, stream: usize, buf: &[u8]) -> Result<usize, PhonicError> {
        self.deref_mut().write(stream, buf)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.deref_mut().flush()
    }

    fn finalize(&mut self) -> Result<(), PhonicError> {
        self.deref_mut().finalize()
    }
}

impl<T> FormatSeeker for T
where
    T: DerefMut,
    T::Target: FormatSeeker,
{
    fn seek(&mut self, stream: usize, offset: i64) -> Result<(), PhonicError> {
        self.deref_mut().seek(stream, offset)
    }
}

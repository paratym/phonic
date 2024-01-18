use crate::{
    io::{
        formats::{FormatTag, SyphonFormat},
        utils::StreamSelector,
        StreamSpec,
    },
    SyphonError,
};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct FormatData<F: FormatTag = SyphonFormat> {
    pub streams: Vec<StreamSpec<F::Codec>>,
}

impl<F: FormatTag> FormatData<F> {
    pub fn new() -> Self {
        Self {
            streams: Vec::new(),
        }
    }

    pub fn with_stream(mut self, spec: StreamSpec<F::Codec>) -> Self {
        self.streams.push(spec);
        self
    }
}

pub trait Format {
    type Tag: FormatTag;

    fn data(&self) -> &FormatData<Self::Tag>;

    fn default_stream(&self) -> Option<usize> {
        self.data()
            .streams
            .iter()
            .position(|spec| spec.codec.is_some())
    }

    fn as_stream(&mut self, i: usize) -> Result<StreamSelector<&mut Self>, SyphonError>
    where
        Self: Sized,
    {
        StreamSelector::new(self, i)
    }

    fn as_default_stream(&mut self) -> Result<StreamSelector<&mut Self>, SyphonError>
    where
        Self: Sized,
    {
        self.default_stream()
            .ok_or(SyphonError::NotFound)
            .and_then(|i| self.as_stream(i))
    }

    fn into_stream(self, i: usize) -> Result<StreamSelector<Self>, SyphonError>
    where
        Self: Sized,
    {
        StreamSelector::new(self, i)
    }

    fn into_default_stream(self) -> Result<StreamSelector<Self>, SyphonError>
    where
        Self: Sized,
    {
        self.default_stream()
            .ok_or(SyphonError::NotFound)
            .and_then(|i| self.into_stream(i))
    }
}
pub enum FormatChunk<'a> {
    Stream { stream_i: usize, buf: &'a [u8] },
}

pub trait FormatReader: Format {
    fn read<'a>(&mut self, buf: &'a mut [u8]) -> Result<FormatChunk<'a>, SyphonError>;
}

pub trait FormatWriter: Format {
    fn write(&mut self, chunk: FormatChunk) -> Result<(), SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;
}

impl<T, F> Format for T
where
    T: Deref,
    T::Target: Format<Tag = F>,
    F: FormatTag,
{
    type Tag = F;

    fn data(&self) -> &FormatData<Self::Tag> {
        self.deref().data()
    }
}

impl<T> FormatReader for T
where
    T: DerefMut,
    T::Target: FormatReader,
{
    fn read<'a>(&mut self, buf: &'a mut [u8]) -> Result<FormatChunk<'a>, SyphonError> {
        self.deref_mut().read(buf)
    }
}

impl<T> FormatWriter for T
where
    T: DerefMut,
    T::Target: FormatWriter,
{
    fn write(&mut self, chunk: FormatChunk) -> Result<(), SyphonError> {
        self.deref_mut().write(chunk)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.deref_mut().flush()
    }
}

use crate::{
    io::{
        formats::{FormatTag, SyphonFormat},
        utils::StreamSelector,
        StreamSpec,
    },
    SyphonError,
};
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone)]
pub struct FormatData<F: FormatTag = SyphonFormat> {
    pub format: Option<F>,
    pub streams: Vec<StreamSpec<F::Codec>>,
}

impl<F: FormatTag> FormatData<F> {
    pub fn new() -> Self {
        Self {
            format: None,
            streams: Vec::new(),
        }
    }

    pub fn with_format(mut self, format: F) -> Self {
        self.format = Some(format);
        self
    }

    pub fn with_stream(mut self, spec: StreamSpec<F::Codec>) -> Self {
        self.streams.push(spec);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.streams.is_empty()
    }

    pub fn merge(&mut self, other: &Self) -> Result<(), SyphonError> {
        if let Some(format) = other.format {
            if self.format.get_or_insert(format) != &format {
                return Err(SyphonError::SignalMismatch);
            }
        }

        let mut other_streams = other.streams.iter();
        for (spec, other) in self.streams.iter_mut().zip(&mut other_streams) {
            spec.merge(*other)?;
        }

        self.streams.extend(other_streams);
        Ok(())
    }

    pub fn fill(&mut self) -> Result<(), SyphonError> {
        F::fill_data(self)
    }

    pub fn filled(mut self) -> Result<Self, SyphonError> {
        self.fill()?;
        Ok(self)
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
    fn read_data(&mut self) -> Result<(), SyphonError>;
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Result<FormatChunk<'a>, SyphonError>;
}

pub trait FormatWriter: Format {
    fn write_data(&mut self, data: &FormatData) -> Result<(), SyphonError>;
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
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Result<FormatChunk<'a>, SyphonError> {
        self.deref_mut().read(buf)
    }

    fn read_data(&mut self) -> Result<(), SyphonError> {
        self.deref_mut().read_data()
    }
}

impl<T> FormatWriter for T
where
    T: DerefMut,
    T::Target: FormatWriter,
{
    fn write_data(&mut self, data: &FormatData) -> Result<(), SyphonError> {
        self.deref_mut().write_data(data)
    }

    fn write(&mut self, chunk: FormatChunk) -> Result<(), SyphonError> {
        self.deref_mut().write(chunk)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.deref_mut().flush()
    }
}

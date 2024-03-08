use crate::{utils::StreamSelector, CodecTag, StreamSpec};
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use syphon_core::SyphonError;

pub trait FormatTag: Sized + Eq + Copy {
    type Codec: CodecTag;

    fn fill_data(data: &mut FormatData<Self>) -> Result<(), SyphonError>;
}

#[derive(Debug, Clone)]
pub struct FormatData<F: FormatTag> {
    pub format: Option<F>,
    pub streams: Vec<StreamSpec<F::Codec>>,
}

#[derive(Debug, Clone, Copy)]
pub struct FormatPosition {
    pub stream_i: usize,
    pub byte_i: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct FormatOffset {
    pub stream_offset: isize,
    pub byte_offset: i64,
}

#[derive(Debug, Clone, Copy)]
pub enum FormatChunk<'a> {
    Stream { stream_i: usize, buf: &'a [u8] },
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

pub trait FormatObserver: Format {
    fn position(&self) -> Result<FormatPosition, SyphonError>;
}

pub trait FormatReader: Format {
    fn read_data(&mut self) -> Result<(), SyphonError>;
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Result<FormatChunk<'a>, SyphonError>;
}

pub trait FormatWriter: Format {
    fn write_data(&mut self, data: &FormatData<Self::Tag>) -> Result<(), SyphonError>;
    fn write(&mut self, chunk: FormatChunk) -> Result<(), SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;
}

pub trait FormatSeeker: Format {
    fn seek(&mut self, offset: FormatOffset) -> Result<(), SyphonError>;

    fn set_position(&mut self, position: FormatPosition) -> Result<(), SyphonError>
    where
        Self: Sized + FormatObserver,
    {
        let current_pos = self.position()?;
        self.seek(FormatOffset {
            stream_offset: current_pos.stream_i as isize - position.stream_i as isize,
            byte_offset: current_pos.byte_i as i64 - position.byte_i as i64,
        })
    }
}

pub trait DynFormat: Format + FormatObserver + FormatReader + FormatWriter + FormatSeeker {}
impl<T> DynFormat for T where T: Format + FormatObserver + FormatReader + FormatWriter + FormatSeeker
{}

impl<F: FormatTag> FormatData<F> {
    pub fn new() -> Self {
        Self {
            format: None,
            streams: Vec::new(),
        }
    }

    pub fn with_tag_type<T>(self) -> FormatData<T>
    where
        T: FormatTag,
        F: TryInto<T>,
        F::Codec: TryInto<T::Codec>,
    {
        FormatData {
            format: self.format.and_then(|f| f.try_into().ok()),
            streams: self
                .streams
                .into_iter()
                .map(StreamSpec::with_tag_type)
                .collect(),
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

impl<T> FormatObserver for T
where
    T: Deref,
    T::Target: FormatObserver,
{
    fn position(&self) -> Result<FormatPosition, SyphonError> {
        self.deref().position()
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
    fn write_data(&mut self, data: &FormatData<Self::Tag>) -> Result<(), SyphonError> {
        self.deref_mut().write_data(data)
    }

    fn write(&mut self, chunk: FormatChunk) -> Result<(), SyphonError> {
        self.deref_mut().write(chunk)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.deref_mut().flush()
    }
}

impl<T> FormatSeeker for T
where
    T: DerefMut,
    T::Target: FormatSeeker,
{
    fn seek(&mut self, offset: FormatOffset) -> Result<(), SyphonError> {
        self.deref_mut().seek(offset)
    }
}

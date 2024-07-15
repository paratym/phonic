use crate::{utils::StreamSelector, CodecTag, StreamSpec};
use phonic_core::PhonicError;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

pub trait FormatTag: Debug + Sized + Eq + Copy + Send + Sync {
    type Codec: CodecTag;

    fn fill_data(data: &mut FormatData<Self>) -> Result<(), PhonicError>;
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
pub enum FormatChunk<'a, F: FormatTag> {
    Data { data: &'a FormatData<F> },
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

pub trait FormatObserver: Format {
    fn position(&self) -> Result<FormatPosition, PhonicError>;
}

pub trait FormatReader: Format {
    fn read<'a>(&'a mut self, buf: &'a mut [u8])
        -> Result<FormatChunk<'a, Self::Tag>, PhonicError>;

    fn with_reader_data(mut self) -> Result<Self, PhonicError>
    where
        Self: Sized,
    {
        if self.data().streams.len() > 0 {
            return Ok(self);
        }

        let mut buf = [0; 512];

        loop {
            match self.read(&mut buf)? {
                FormatChunk::Data { data } if data.streams.len() > 0 => return Ok(self),
                FormatChunk::Stream { .. } => return Err(PhonicError::InvalidData),
                _ => continue,
            }
        }
    }
}

pub trait FormatWriter: Format {
    fn write(&mut self, chunk: FormatChunk<Self::Tag>) -> Result<(), PhonicError>;
    fn flush(&mut self) -> Result<(), PhonicError>;
}

pub trait FormatSeeker: Format {
    fn seek(&mut self, offset: FormatOffset) -> Result<(), PhonicError>;

    fn set_position(&mut self, position: FormatPosition) -> Result<(), PhonicError>
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

    pub fn merge(&mut self, other: &Self) -> Result<(), PhonicError> {
        if let Some(format) = other.format {
            if self.format.get_or_insert(format) != &format {
                return Err(PhonicError::SignalMismatch);
            }
        }

        let mut other_streams = other.streams.iter();
        for (spec, other) in self.streams.iter_mut().zip(&mut other_streams) {
            spec.merge(*other)?;
        }

        self.streams.extend(other_streams);
        Ok(())
    }

    pub fn fill(&mut self) -> Result<(), PhonicError> {
        F::fill_data(self)
    }

    pub fn filled(mut self) -> Result<Self, PhonicError> {
        self.fill()?;
        Ok(self)
    }
}

impl<'a, F: FormatTag> From<&'a FormatData<F>> for FormatChunk<'a, F> {
    fn from(data: &'a FormatData<F>) -> Self {
        FormatChunk::Data { data }
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
    fn position(&self) -> Result<FormatPosition, PhonicError> {
        self.deref().position()
    }
}

impl<T> FormatReader for T
where
    T: DerefMut,
    T::Target: FormatReader,
{
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> Result<FormatChunk<'a, Self::Tag>, PhonicError> {
        self.deref_mut().read(buf)
    }
}

impl<T> FormatWriter for T
where
    T: DerefMut,
    T::Target: FormatWriter,
{
    fn write(&mut self, chunk: FormatChunk<Self::Tag>) -> Result<(), PhonicError> {
        self.deref_mut().write(chunk)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.deref_mut().flush()
    }
}

impl<T> FormatSeeker for T
where
    T: DerefMut,
    T::Target: FormatSeeker,
{
    fn seek(&mut self, offset: FormatOffset) -> Result<(), PhonicError> {
        self.deref_mut().seek(offset)
    }
}

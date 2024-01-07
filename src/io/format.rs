use crate::{
    io::{utils::Track, StreamSpec, StreamSpecBuilder, SyphonFormat},
    SyphonError,
};
use std::{
    io::{Read, Seek, Write},
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone)]
pub struct FormatData {
    pub format: Option<SyphonFormat>,
    pub tracks: Vec<StreamSpecBuilder>,
}

impl FormatData {
    pub fn new() -> Self {
        Self {
            format: None,
            tracks: Vec::new(),
        }
    }

    pub fn with_format(mut self, format: SyphonFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn with_track(mut self, track: StreamSpecBuilder) -> Self {
        self.tracks.push(track);
        self
    }

    pub fn filled(self) -> Result<Self, SyphonError> {
        SyphonFormat::fill_data(self)
    }
}

pub trait Format {
    fn data(&self) -> &FormatData;

    fn default_track(&self) -> Option<usize> {
        self.data().tracks.iter().position(|t| t.codec.is_some())
    }

    fn as_track(&mut self, i: usize) -> Result<Track<&mut Self>, SyphonError>
    where
        Self: Sized,
    {
        Track::new(self, i)
    }

    fn as_default_track(&mut self) -> Result<Track<&mut Self>, SyphonError>
    where
        Self: Sized,
    {
        self.default_track()
            .ok_or(SyphonError::NotFound)
            .and_then(|i| self.as_track(i))
    }

    fn into_track(self, i: usize) -> Result<Track<Self>, SyphonError>
    where
        Self: Sized,
    {
        Track::new(self, i)
    }

    fn into_default_track(self) -> Result<Track<Self>, SyphonError>
    where
        Self: Sized,
    {
        self.default_track()
            .ok_or(SyphonError::NotFound)
            .and_then(|i| self.into_track(i))
    }
}

pub struct TrackChunk<'a> {
    pub i: usize,
    pub buf: &'a [u8],
}

pub enum FormatChunk<'a> {
    Track(TrackChunk<'a>),
}

pub trait FormatReader: Format {
    fn read<'a>(&mut self, buf: &'a mut [u8]) -> Result<FormatChunk<'a>, SyphonError>;
}

pub trait FormatWriter: Format {
    fn write_track_chunk(&mut self, chunk: TrackChunk) -> Result<usize, SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;
}

impl<T> Format for T
where
    T: Deref,
    T::Target: Format,
{
    fn data(&self) -> &FormatData {
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
    fn write_track_chunk(&mut self, chunk: TrackChunk) -> Result<usize, SyphonError> {
        self.deref_mut().write_track_chunk(chunk)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.deref_mut().flush()
    }
}

pub trait IntoFormatReader {
    fn into_format_reader(self, format: SyphonFormat)
        -> Result<Box<dyn FormatReader>, SyphonError>;
}

pub trait ResolveFormatReader {
    fn resolve_format_reader(
        self,
        identifier: Option<impl TryInto<SyphonFormat>>,
    ) -> Result<Box<dyn FormatReader>, SyphonError>;
}

pub trait IntoFormatWriter {
    fn into_format_writer(self, data: FormatData) -> Result<Box<dyn FormatWriter>, SyphonError>;
}

impl<T: Read + 'static> IntoFormatReader for T {
    fn into_format_reader(
        self,
        format: SyphonFormat,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        format.construct_reader(self)
    }
}

impl<T: Read + Seek + 'static> ResolveFormatReader for T {
    fn resolve_format_reader(
        mut self,
        identifier: Option<impl TryInto<SyphonFormat>>,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        let format = identifier
            .and_then(|f| f.try_into().ok())
            .or_else(|| SyphonFormat::resolve(&mut self).ok())
            .ok_or(SyphonError::Unsupported)?;

        self.into_format_reader(format)
    }
}

impl<T: Write + 'static> IntoFormatWriter for T {
    fn into_format_writer(self, data: FormatData) -> Result<Box<dyn FormatWriter>, SyphonError> {
        SyphonFormat::construct_writer(data, self)
    }
}

use crate::{
    io::{
        formats::{KnownFormat, SyphonFormat},
        utils::Track,
        StreamSpec,
    },
    SyphonError,
};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub struct FormatData<F: KnownFormat = SyphonFormat> {
    pub tracks: Vec<(Option<F::Codec>, StreamSpec)>,
}

impl<F: KnownFormat> FormatData<F> {
    pub fn new() -> Self {
        Self { tracks: Vec::new() }
    }

    pub fn with_track(mut self, codec: Option<F::Codec>, spec: StreamSpec) -> Self {
        self.tracks.push((codec, spec));
        self
    }
}

pub trait Format {
    type Format: KnownFormat;

    fn format(&self) -> Option<Self::Format>;
    fn data(&self) -> &FormatData<Self::Format>;

    fn default_track(&self) -> Option<usize> {
        self.data().tracks.iter().position(|(c, _)| c.is_some())
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

impl<T, F> Format for T
where
    T: Deref,
    T::Target: Format<Format = F>,
    F: KnownFormat,
{
    type Format = F;

    fn format(&self) -> Option<Self::Format> {
        self.deref().format()
    }

    fn data(&self) -> &FormatData<F> {
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

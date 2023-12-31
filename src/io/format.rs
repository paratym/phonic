use crate::{
    io::{utils::Track, StreamSpec, StreamSpecBuilder, SyphonCodec, SyphonFormat},
    SyphonError,
};
use std::{io::Write, ops::{Deref, DerefMut}};

#[derive(Debug, Clone)]
pub struct FormatData {
    pub format: SyphonFormat,
    pub tracks: Box<[StreamSpec]>,
}

pub struct FormatDataBuilder {
    pub format: Option<SyphonFormat>,
    pub tracks: Vec<StreamSpecBuilder>,
}

impl FormatData {
    pub fn builder() -> FormatDataBuilder {
        FormatDataBuilder::new()
    }

    pub fn construct_writer(self, sink: Box<dyn Write>) -> Result<Box<dyn FormatWriter>, SyphonError> {
        let format = self.format;
        format.construct_writer(sink, self)
    }
}

impl TryFrom<FormatDataBuilder> for FormatData {
    type Error = SyphonError;

    fn try_from(builder: FormatDataBuilder) -> Result<Self, Self::Error> {
        Ok(Self {
            format: builder.format.unwrap_or(SyphonFormat::Unknown),
            tracks: builder
                .tracks
                .into_iter()
                .map(|track| track.try_into())
                .collect::<Result<_, _>>()?,
        })
    }
}

impl FormatDataBuilder {
    pub fn new() -> Self {
        Self {
            format: None,
            tracks: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.format.is_none() && self.tracks.is_empty()
    }

    pub fn with_format(mut self, format: impl Into<Option<SyphonFormat>>) -> Self {
        self.format = format.into();
        self
    }

    pub fn with_track(mut self, track: impl Into<StreamSpecBuilder>) -> Self {
        self.tracks.push(track.into());
        self
    }

    pub fn fill(&mut self) -> Result<(), SyphonError> {
        if let Some(format) = self.format {
            format.fill_data(self)?;
        }

        for track in &mut self.tracks {
            track.fill()?;
        }

        Ok(())
    }

    pub fn filled(mut self) -> Result<Self, SyphonError> {
        self.fill()?;
        Ok(self)
    }

    pub fn build(self) -> Result<FormatData, SyphonError> {
        self.try_into()
    }
}

pub trait Format {
    fn format_data(&self) -> &FormatData;

    fn default_track(&self) -> Result<usize, SyphonError> {
        self.format_data()
            .tracks
            .iter()
            .position(|track| track.codec != SyphonCodec::Unknown)
            .ok_or(SyphonError::NotFound)
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
        self.default_track().and_then(|i| self.as_track(i))
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
        self.default_track().and_then(|i| self.into_track(i))
    }
}

pub struct FormatReadResult {
    pub track: usize,
    pub n: usize,
}

pub trait FormatReader: Format {
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError>;
}

pub trait FormatWriter: Format {
    fn write(&mut self, track_i: usize, buf: &[u8]) -> Result<usize, SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;
}

impl<T> Format for T
where
    T: Deref,
    T::Target: Format,
{
    fn format_data(&self) -> &FormatData {
        self.deref().format_data()
    }
}

impl<T> FormatReader for T
where
    T: DerefMut,
    T::Target: FormatReader,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError> {
        self.deref_mut().read(buf)
    }
}

impl<T> FormatWriter for T
where
    T: DerefMut,
    T::Target: FormatWriter,
{
    fn write(&mut self, track_i: usize, buf: &[u8]) -> Result<usize, SyphonError> {
        self.deref_mut().write(track_i, buf)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.deref_mut().flush()
    }
}
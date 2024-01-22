use crate::{
    io::{
        codecs::CodecTag,
        codecs::SyphonCodec,
        formats::wave::{WaveFormat, WAVE_IDENTIFIERS},
        FormatData, FormatReader, FormatWriter,
    },
    SyphonError,
};
use std::{
    hash::Hash,
    io::{Read, Write},
    path::Path,
};

use super::wave::fill_wave_format_data;

pub trait FormatTag: Sized + Eq + Copy {
    type Codec: CodecTag;

    fn fill_data(data: &mut FormatData<Self>) -> Result<(), SyphonError>;

    fn demux_reader(
        &self,
        inner: impl Read + 'static,
    ) -> Result<Box<dyn FormatReader<Tag = Self>>, SyphonError>;

    fn mux_writer(
        &self,
        inner: impl Write + 'static,
    ) -> Result<Box<dyn FormatWriter<Tag = Self>>, SyphonError>;

    fn mux_reader(
        reader: impl FormatReader<Tag = Self> + 'static,
    ) -> Result<Box<dyn Read>, SyphonError>;

    fn demux_writer(
        writer: impl FormatWriter<Tag = Self> + 'static,
    ) -> Result<Box<dyn Write>, SyphonError>;
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum SyphonFormat {
    Wave,
}

impl SyphonFormat {
    pub fn all() -> &'static [Self] {
        const SYPHON_FORMATS: &[SyphonFormat] = &[SyphonFormat::Wave];
        SYPHON_FORMATS
    }

    pub fn identifiers(&self) -> &'static FormatIdentifiers {
        match self {
            &Self::Wave => &WAVE_IDENTIFIERS,
        }
    }
}

impl FormatTag for SyphonFormat {
    type Codec = SyphonCodec;

    fn fill_data(data: &mut FormatData<Self>) -> Result<(), SyphonError> {
        match data.format {
            Some(Self::Wave) => fill_wave_format_data(data),
            None => Ok(()), 
        }
    }

    fn demux_reader(
        &self,
        inner: impl Read + 'static,
    ) -> Result<Box<dyn FormatReader<Tag = Self>>, SyphonError> {
        Ok(match self {
            SyphonFormat::Wave => Box::new(WaveFormat::new(inner)?),
        })
    }

    fn mux_writer(
        &self,
        inner: impl Write + 'static,
    ) -> Result<Box<dyn FormatWriter<Tag = Self>>, SyphonError> {
        Ok(match self {
            SyphonFormat::Wave => Box::new(WaveFormat::new(inner)?),
        })
    }

    fn mux_reader(
        inner: impl FormatReader<Tag = Self> + 'static,
    ) -> Result<Box<dyn Read>, SyphonError> {
        todo!()
    }

    fn demux_writer(
        inner: impl FormatWriter<Tag = Self> + 'static,
    ) -> Result<Box<dyn Write>, SyphonError> {
        todo!()
    }
}

pub struct FormatIdentifiers {
    pub file_extensions: &'static [&'static str],
    pub mime_types: &'static [&'static str],
    pub markers: &'static [&'static [u8]],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FormatIdentifier<'a> {
    FileExtension(&'a str),
    MimeType(&'a str),
}

impl<'a> TryFrom<&FormatIdentifier<'a>> for SyphonFormat {
    type Error = SyphonError;

    fn try_from(id: &FormatIdentifier<'a>) -> Result<Self, Self::Error> {
        Self::all()
            .iter()
            .find(|fmt| fmt.identifiers().contains(id))
            .copied()
            .ok_or(SyphonError::Unsupported)
    }
}

impl FormatIdentifiers {
    fn contains(&self, identifier: &FormatIdentifier) -> bool {
        match identifier {
            FormatIdentifier::FileExtension(ext) => self.file_extensions.contains(ext),
            FormatIdentifier::MimeType(mime) => self.mime_types.contains(mime),
        }
    }
}

impl<'a> TryFrom<&'a Path> for FormatIdentifier<'a> {
    type Error = SyphonError;

    fn try_from(path: &'a Path) -> Result<Self, Self::Error> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| FormatIdentifier::FileExtension(ext))
            .ok_or(SyphonError::MissingData)
    }
}

use crate::{
    io::{
        format,
        formats::wave::{WaveFormat, WAVE_IDENTIFIERS},
        FormatData, FormatDataBuilder, FormatReader, FormatWriter,
    },
    SyphonError,
};
use std::{
    hash::Hash,
    io::{Read, Seek, Write},
};

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum SyphonFormat {
    Wave,
    Unknown,
}

pub struct FormatIdentifiers {
    pub file_extensions: &'static [&'static str],
    pub mime_types: &'static [&'static str],
    pub markers: &'static [&'static [u8]],
}

#[derive(Clone, Copy)]
pub enum FormatIdentifier<'a> {
    None,
    FileExtension(&'a str),
    MimeType(&'a str),
}

impl SyphonFormat {
    pub fn all() -> &'static [Self] {
        const SYPHON_FORMATS: &[SyphonFormat] = &[SyphonFormat::Wave];
        SYPHON_FORMATS
    }

    pub fn identifiers(&self) -> &'static FormatIdentifiers {
        const UNKNOWN_IDENTIFIERS: FormatIdentifiers = FormatIdentifiers {
            file_extensions: &[],
            mime_types: &[],
            markers: &[],
        };

        match self {
            SyphonFormat::Wave => &WAVE_IDENTIFIERS,
            SyphonFormat::Unknown => &UNKNOWN_IDENTIFIERS,
        }
    }

    pub fn construct_reader(
        &self,
        inner: impl Read + Seek + 'static,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        Ok(match self {
            SyphonFormat::Wave => Box::new(WaveFormat::read(inner)?.into_format()?),
            SyphonFormat::Unknown => return Err(SyphonError::Unsupported),
        })
    }

    pub fn construct_writer(
        &self,
        inner: impl Write + 'static,
        mut data: FormatDataBuilder,
    ) -> Result<Box<dyn FormatWriter>, SyphonError> {
        if data.format.is_some_and(|f| f != *self) {
            return Err(SyphonError::InvalidData);
        }

        data = data.with_format(*self);

        Ok(match self {
            SyphonFormat::Wave => {
                Box::new(WaveFormat::write(inner, data.try_into()?)?.into_format()?)
            }
            SyphonFormat::Unknown => return Err(SyphonError::Unsupported),
        })
    }

    pub fn resolve(source: &mut (impl Read + Seek)) -> Result<Self, SyphonError> {
        todo!()
    }
}

impl Default for SyphonFormat {
    fn default() -> Self {
        SyphonFormat::Unknown
    }
}

impl<'a> From<&FormatIdentifier<'a>> for SyphonFormat {
    fn from(identifier: &FormatIdentifier) -> Self {
        Self::all()
            .iter()
            .find(|fmt| fmt.identifiers().contains(identifier))
            .copied()
            .unwrap_or_default()
    }
}

impl FormatIdentifiers {
    fn contains(&self, identifier: &FormatIdentifier) -> bool {
        match identifier {
            FormatIdentifier::FileExtension(ext) => self.file_extensions.contains(ext),
            FormatIdentifier::MimeType(mime) => self.mime_types.contains(mime),
            FormatIdentifier::None => false,
        }
    }
}

impl<'a> Default for FormatIdentifier<'a> {
    fn default() -> Self {
        FormatIdentifier::None
    }
}
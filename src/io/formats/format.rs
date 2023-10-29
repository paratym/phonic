use crate::{
    io::{
        formats::wave::{fill_wave_data, Wave, WAVE_IDENTIFIERS},
        FormatData, FormatDataBuilder, FormatReader, FormatWriter,
    },
    SyphonError,
};
use std::{
    hash::Hash,
    io::{Read, Write},
};

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum SyphonFormat {
    Wave,
    Unknown,
}

impl SyphonFormat {
    pub fn iter() -> impl Iterator<Item = SyphonFormat> {
        const SYPHON_FORMATS: &[SyphonFormat] = &[SyphonFormat::Wave];
        SYPHON_FORMATS.iter().copied()
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

    pub fn fill_data(&self, data: &mut FormatDataBuilder) -> Result<(), SyphonError> {
        match self {
            SyphonFormat::Wave => fill_wave_data(data),
            SyphonFormat::Unknown => Ok(()),
        }
    }

    pub fn reader(&self, source: Box<dyn Read>) -> Result<Box<dyn FormatReader>, SyphonError> {
        match self {
            SyphonFormat::Wave => Ok(Box::new(Wave::read(source)?.into_format()?)),
            SyphonFormat::Unknown => Err(SyphonError::Unsupported),
        }
    }

    pub fn writer(
        &self,
        sink: Box<dyn Write>,
        data: FormatData,
    ) -> Result<Box<dyn FormatWriter>, SyphonError> {
        match self {
            SyphonFormat::Wave => Ok(Box::new(
                Wave::write(sink, data.try_into()?)?.into_format()?,
            )),
            SyphonFormat::Unknown => Err(SyphonError::Unsupported),
        }
    }

    fn resolve_from_reader(reader: &mut dyn Read) -> Option<Self> {
        None
    }

    pub fn resolve_reader(
        mut source: Box<dyn Read>,
        identifier: Option<FormatIdentifier>,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        identifier
            .and_then(|id| Self::iter().find(|fmt| fmt.identifiers().contains(&id)))
            .or_else(|| SyphonFormat::resolve_from_reader(&mut source))
            .ok_or(SyphonError::Unsupported)?
            .reader(source)
    }
}

pub struct FormatIdentifiers {
    pub file_extensions: &'static [&'static str],
    pub mime_types: &'static [&'static str],
    pub markers: &'static [&'static [u8]],
}

#[derive(Clone, Copy)]
pub enum FormatIdentifier<'a> {
    FileExtension(&'a str),
    MimeType(&'a str),
}

impl FormatIdentifiers {
    fn contains(&self, identifier: &FormatIdentifier) -> bool {
        match identifier {
            FormatIdentifier::FileExtension(ext) => self.file_extensions.contains(ext),
            FormatIdentifier::MimeType(mime) => self.mime_types.contains(mime),
        }
    }
}

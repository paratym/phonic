use crate::{
    io::{
        formats::wave::{fill_wave_data, Wave, WAVE_IDENTIFIERS},
        FormatData, FormatReader, FormatWriter,
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
}

const SYPHON_CODECS: &[SyphonFormat] = &[SyphonFormat::Wave];

impl SyphonFormat {
    pub fn identifiers(&self) -> &'static FormatIdentifiers {
        match self {
            SyphonFormat::Wave => &WAVE_IDENTIFIERS,
        }
    }

    pub fn fill_data(&self, data: &mut FormatData) -> Result<(), SyphonError> {
        match self {
            SyphonFormat::Wave => fill_wave_data(data),
        }
    }

    pub fn reader(&self, source: Box<dyn Read>) -> Result<Box<dyn FormatReader>, SyphonError> {
        match self {
            SyphonFormat::Wave => Ok(Box::new(Wave::read(source)?.into_format())),
        }
    }

    pub fn writer(
        &self,
        sink: Box<dyn Write>,
        data: FormatData,
    ) -> Result<Box<dyn FormatWriter>, SyphonError> {
        match self {
            SyphonFormat::Wave => Ok(Box::new(Wave::write(sink, data.try_into()?)?.into_format())),
        }
    }

    pub fn resolve_from_reader(reader: &mut dyn Read) -> Option<Self> {
        None
    }

    pub fn resolve_reader(
        mut source: Box<dyn Read>,
        identifier: Option<FormatIdentifier>,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        identifier
            .and_then(|id| {
                SYPHON_CODECS
                    .iter()
                    .find(|fmt| fmt.identifiers().contains(&id))
                    .copied()
            })
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

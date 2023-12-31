use crate::{
    io::{
        formats::wave::{fill_wave_data, Wave, WAVE_IDENTIFIERS},
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

    pub fn fill_data(&self, data: &mut FormatDataBuilder) -> Result<(), SyphonError> {
        match self {
            SyphonFormat::Wave => fill_wave_data(data),
            SyphonFormat::Unknown => Ok(()),
        }
    }

    pub fn construct_reader(
        &self,
        source: impl Read + 'static,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        Ok(match self {
            SyphonFormat::Wave => Box::new(Wave::read(source)?.into_format()?),
            SyphonFormat::Unknown => return Err(SyphonError::Unsupported),
        })
    }

    pub fn construct_writer(
        &self,
        inner: impl Write + 'static,
        data: FormatData,
    ) -> Result<Box<dyn FormatWriter>, SyphonError> {
        Ok(match self {
            SyphonFormat::Wave => Box::new(Wave::write(inner, data.try_into()?)?.into_format()?),
            SyphonFormat::Unknown => return Err(SyphonError::Unsupported),
        })
    }

    pub fn resolve(
        mut source: impl Read + Seek,
        identifier: Option<&FormatIdentifier>,
    ) -> Result<Self, SyphonError> {
        if let Some(id) = identifier {
            return Self::all()
                .iter()
                .find(|fmt| fmt.identifiers().contains(id))
                .copied()
                .ok_or(SyphonError::Unsupported);
        }

        todo!()
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

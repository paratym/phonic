use crate::{
    io::{
        formats::wave::{WaveFormat, WAVE_IDENTIFIERS},
        FormatData, FormatReader, FormatWriter, StreamSpec, SyphonCodec,
    },
    SyphonError,
};
use std::{
    hash::Hash,
    io::{Read, Seek, Write},
};

use super::wave::fill_wave_format_data;

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum SyphonFormat {
    Wave,
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

impl SyphonFormat {
    pub fn all() -> &'static [Self] {
        const SYPHON_FORMATS: &[SyphonFormat] = &[SyphonFormat::Wave];
        SYPHON_FORMATS
    }

    pub fn identifiers(&self) -> &'static FormatIdentifiers {
        match self {
            SyphonFormat::Wave => &WAVE_IDENTIFIERS,
        }
    }

    pub fn fill_data(mut data: FormatData) -> Result<FormatData, SyphonError> {
        data = match data.format.ok_or(SyphonError::MissingData)? {
            SyphonFormat::Wave => fill_wave_format_data(data)?,
        };

        data.tracks = data
            .tracks
            .into_iter()
            .map(StreamSpec::filled)
            .collect::<Result<_, _>>()?;

        Ok(data)
    }

    pub fn resolve(source: &mut (impl Read + Seek)) -> Result<Self, SyphonError> {
        todo!()
    }

    pub fn construct_reader(
        &self,
        inner: impl Read + 'static,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        Ok(match self {
            SyphonFormat::Wave => Box::new(WaveFormat::read(inner)?.into_format()?),
        })
    }

    pub fn construct_writer(
        data: FormatData,
        inner: impl Write + 'static,
    ) -> Result<Box<dyn FormatWriter>, SyphonError> {
        Ok(match data.format.ok_or(SyphonError::MissingData)? {
            SyphonFormat::Wave => {
                Box::new(WaveFormat::write(inner, data.try_into()?)?.into_format()?)
            }
        })
    }
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

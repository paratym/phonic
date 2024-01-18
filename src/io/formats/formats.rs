use crate::{
    io::{
        codecs::CodecTag,
        codecs::SyphonCodec,
        formats::wave::{fill_wave_format_data, WaveFormat, WAVE_IDENTIFIERS},
        FormatData, FormatReader, FormatWriter,
    },
    SyphonError,
};
use std::{
    hash::Hash,
    io::{Read, Seek, Write},
    path::Path,
};

pub trait FormatTag: Sized + Hash + Eq + Copy {
    type Codec: CodecTag;

    fn init() -> Result<(), SyphonError>;

    fn fill_data(&self, data: &mut FormatData<Self>) -> Result<(), SyphonError>;

    fn construct_reader(
        &self,
        inner: impl Read + 'static,
    ) -> Result<Box<dyn FormatReader<Tag = Self>>, SyphonError>;

    fn construct_writer(
        &self,
        inner: impl Write + 'static,
        data: FormatData<Self>,
    ) -> Result<Box<dyn FormatWriter<Tag = Self>>, SyphonError>;

    fn resolve(source: &mut (impl Read + Seek)) -> Result<Self, SyphonError>;

    fn resolve_reader<T, I>(
        mut source: T,
        identifier: Option<I>,
    ) -> Result<Box<dyn FormatReader<Tag = Self>>, SyphonError>
    where
        T: Read + Seek + 'static,
        I: TryInto<Self>,
        I::Error: Into<SyphonError>,
    {
        identifier
            .ok_or(SyphonError::MissingData)
            .and_then(|id| id.try_into().map_err(Into::into))
            .or_else(|_| Self::resolve(&mut source))?
            .construct_reader(source)
    }
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
            SyphonFormat::Wave => &WAVE_IDENTIFIERS,
        }
    }
}

impl FormatTag for SyphonFormat {
    type Codec = SyphonCodec;

    fn init() -> Result<(), SyphonError> {
        Ok(())
    }

    fn fill_data(&self, data: &mut FormatData<Self>) -> Result<(), SyphonError> {
        match self {
            SyphonFormat::Wave => fill_wave_format_data(data)?,
        };

        for (codec, spec) in data.streams.iter_mut() {
            if let Some(codec) = codec {
                codec.fill_spec(spec)?;
            }
        }

        Ok(())
    }

    fn resolve(source: &mut (impl Read + Seek)) -> Result<Self, SyphonError> {
        todo!()
    }

    fn construct_reader(
        &self,
        inner: impl Read + 'static,
    ) -> Result<Box<dyn FormatReader<Tag = Self>>, SyphonError> {
        Ok(match self {
            SyphonFormat::Wave => Box::new(WaveFormat::read(inner)?.into_format()?),
        })
    }

    fn construct_writer(
        &self,
        inner: impl Write + 'static,
        data: FormatData<Self>,
    ) -> Result<Box<dyn FormatWriter<Tag = Self>>, SyphonError> {
        Ok(match self {
            SyphonFormat::Wave => {
                Box::new(WaveFormat::write(inner, data.try_into()?)?.into_format()?)
            }
        })
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

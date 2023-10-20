use crate::{
    io::{
        // formats::{WavReader, WAV_FORMAT_IDENTIFIERS},
        FormatReader,
        MediaSource,
    },
    SyphonError,
};
use std::{collections::HashMap, hash::Hash};

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum SyphonFormat {
    Wav,
    Other(&'static str),
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

pub struct FormatRegistry {
    formats: HashMap<
        SyphonFormat,
        (
            Option<&'static FormatIdentifiers>,
            Option<Box<dyn Fn(Box<dyn MediaSource>) -> Box<dyn FormatReader>>>,
        ),
    >,
}

impl FormatRegistry {
    pub fn new() -> Self {
        Self {
            formats: HashMap::new(),
        }
    }

    pub fn register_identifiers(
        mut self,
        key: SyphonFormat,
        identifiers: &'static FormatIdentifiers,
    ) -> Self {
        let entry = self.formats.entry(key).or_insert((None, None));
        entry.0 = Some(identifiers);

        self
    }

    pub fn register_reader(
        mut self,
        key: SyphonFormat,
        reader_constructor: impl Fn(Box<dyn MediaSource>) -> Box<dyn FormatReader> + 'static,
    ) -> Self {
        let format = self.formats.entry(key).or_insert((None, None));
        format.1 = Some(Box::new(reader_constructor));

        self
    }

    pub fn register_format(
        mut self,
        key: SyphonFormat,
        identifiers: &'static FormatIdentifiers,
        reader_constructor: impl Fn(Box<dyn MediaSource>) -> Box<dyn FormatReader> + 'static,
    ) -> Self {
        self.formats
            .insert(key, (Some(identifiers), Some(Box::new(reader_constructor))));

        self
    }

    pub fn construct_reader(
        &self,
        key: &SyphonFormat,
        source: Box<dyn MediaSource>,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        let constructor = self
            .formats
            .get(key)
            .map(|f| f.1.as_ref())
            .flatten()
            .ok_or(SyphonError::Unsupported)?;

        Ok(constructor(Box::new(source)))
    }

    pub fn resolve_format(
        &self,
        source: &mut impl MediaSource,
    ) -> Result<SyphonFormat, SyphonError> {
        todo!()
    }

    pub fn resolve_reader(
        &self,
        mut source: Box<dyn MediaSource>,
        identifier: Option<FormatIdentifier>,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        let key = if let Some(id) = identifier {
            *self
                .formats
                .iter()
                .find(|(_, (ids, _))| ids.is_some_and(|ids| ids.contains(&id)))
                .map(|(key, _)| key)
                .ok_or(SyphonError::Unsupported)?
        } else {
            self.resolve_format(&mut source)?
        };

        self.construct_reader(&key, source)
    }
}

pub fn syphon_format_registry() -> FormatRegistry {
    FormatRegistry::new()
    // .register_format(SyphonFormat::Wav, &WAV_FORMAT_IDENTIFIERS, |source| {
    //     Box::new(WavReader::new(source))
    // })
}

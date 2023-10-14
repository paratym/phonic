use crate::{
    io::{
        formats::{WavReader, WAV_FORMAT_IDENTIFIERS},
        BufReader, FormatReadResult, FormatReader, MediaSource, StreamSpecBuilder, SyphonCodec,
        UnseekableMediaSource,
    },
    SyphonError,
};
use std::{
    collections::HashMap,
    hash::Hash,
    io::{Read, Seek, SeekFrom},
    marker::PhantomData,
};

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

const MAX_MARKER_INDEX: usize = 1024;

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
        source: impl MediaSource + 'static,
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
        reader: &mut (impl MediaSource + 'static),
        identifier: Option<FormatIdentifier>,
    ) -> Result<SyphonFormat, SyphonError> {
        // self.formats
        //     .iter()
        //     .filter_map(|(key, format)| Some((key, format.0.as_ref()?)))
        //     .find_map(|(k, identifiers)| {
        //         if let Some(hint) = hint {
        //             if hint.matches(identifiers) {
        //                 return Some(*k);
        //             }
        //         }

        //         // TODO: check for markers

        //         None
        //     })

        Err(SyphonError::Unsupported)
    }

    pub fn resolve_reader(
        &self,
        mut reader: impl MediaSource + 'static,
        identifier: Option<FormatIdentifier>,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        let key = self.resolve_format(&mut reader, identifier)?;
        self.construct_reader(&key, Box::new(reader))
    }
}

pub fn syphon_format_registry() -> FormatRegistry {
    FormatRegistry::new().register_format(SyphonFormat::Wav, &WAV_FORMAT_IDENTIFIERS, |reader| {
        Box::new(WavReader::new(reader))
    })
}

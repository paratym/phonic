use crate::{
    io::{
        formats::wav::{WavFormat, WAV_FORMAT_IDENTIFIERS},
        FormatData, FormatReader, FormatWriter,
    },
    SyphonError,
};
use std::{
    collections::HashMap,
    hash::Hash,
    io::{Read, Seek, Write},
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

impl FormatIdentifiers {
    fn contains(&self, identifier: &FormatIdentifier) -> bool {
        match identifier {
            FormatIdentifier::FileExtension(ext) => self.file_extensions.contains(ext),
            FormatIdentifier::MimeType(mime) => self.mime_types.contains(mime),
        }
    }
}

pub trait MediaSource: Read + Seek {}
pub trait MediaSink: Write + Seek {}

impl<T: Read + Seek> MediaSource for T {}
impl<T: Write + Seek> MediaSink for T {}

pub struct FormatRegistry {
    formats: HashMap<
        SyphonFormat,
        (
            Option<&'static FormatIdentifiers>,
            Option<Box<dyn Fn(Box<dyn MediaSource>) -> Result<Box<dyn FormatReader>, SyphonError>>>,
            Option<
                Box<
                    dyn Fn(
                        Box<dyn MediaSink>,
                        FormatData,
                    ) -> Result<Box<dyn FormatWriter>, SyphonError>,
                >,
            >,
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
        let format = self.formats.entry(key).or_insert((None, None, None));
        format.0 = Some(identifiers);

        self
    }

    pub fn register_reader<C>(mut self, key: SyphonFormat, constructor: C) -> Self
    where
        C: Fn(Box<dyn MediaSource>) -> Result<Box<dyn FormatReader>, SyphonError> + 'static,
    {
        let format = self.formats.entry(key).or_insert((None, None, None));
        format.1 = Some(Box::new(constructor));

        self
    }

    pub fn register_writer<C>(mut self, key: SyphonFormat, constructor: C) -> Self
    where
        C: Fn(Box<dyn MediaSink>, FormatData) -> Result<Box<dyn FormatWriter>, SyphonError>
            + 'static,
    {
        let format = self.formats.entry(key).or_insert((None, None, None));
        format.2 = Some(Box::new(constructor));

        self
    }

    pub fn register_format<R, W>(
        mut self,
        key: SyphonFormat,
        identifiers: &'static FormatIdentifiers,
        reader_constructor: R,
        writer_constructor: W,
    ) -> Self
    where
        R: Fn(Box<dyn MediaSource>) -> Result<Box<dyn FormatReader>, SyphonError> + 'static,
        W: Fn(Box<dyn MediaSink>, FormatData) -> Result<Box<dyn FormatWriter>, SyphonError>
            + 'static,
    {
        self.formats.insert(
            key,
            (
                Some(identifiers),
                Some(Box::new(reader_constructor)),
                Some(Box::new(writer_constructor)),
            ),
        );

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

        constructor(source)
    }

    pub fn construct_writer(
        &self,
        key: &SyphonFormat,
        sink: Box<dyn MediaSink>,
        data: FormatData,
    ) -> Result<Box<dyn FormatWriter>, SyphonError> {
        let constructor = self
            .formats
            .get(key)
            .map(|f| f.2.as_ref())
            .flatten()
            .ok_or(SyphonError::Unsupported)?;

        constructor(sink, data)
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
                .find(|(_, (ids, _, _))| ids.is_some_and(|ids| ids.contains(&id)))
                .map(|(key, _)| key)
                .ok_or(SyphonError::Unsupported)?
        } else {
            self.resolve_format(&mut source)?
        };

        self.construct_reader(&key, source)
    }
}

pub fn syphon_format_registry() -> FormatRegistry {
    FormatRegistry::new().register_format(
        SyphonFormat::Wav,
        &WAV_FORMAT_IDENTIFIERS,
        |source| Ok(Box::new(WavFormat::read(source)?.into_dyn_format())),
        |sink, data| {
            Ok(Box::new(
                WavFormat::write(sink, data.try_into()?)?.into_dyn_format(),
            ))
        },
    )
}

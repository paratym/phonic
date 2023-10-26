use crate::{
    io::{
        formats::wave::{fill_wave_data, Wave, WAVE_IDENTIFIERS},
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
    Wave,
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
            Option<Box<dyn Fn(&mut FormatData) -> Result<(), SyphonError>>>,
            Option<Box<dyn Fn(Box<dyn Read>) -> Result<Box<dyn FormatReader>, SyphonError>>>,
            Option<
                Box<
                    dyn Fn(
                        Box<dyn Write>,
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
        let format = self.formats.entry(key).or_insert((None, None, None, None));
        format.0 = Some(identifiers);

        self
    }

    pub fn register_data_filler<F>(&mut self, key: SyphonFormat, filler: F)
    where
        F: Fn(&mut FormatData) -> Result<(), SyphonError> + 'static,
    {
        let format = self.formats.entry(key).or_insert((None, None, None, None));
        format.1 = Some(Box::new(filler));
    }

    pub fn register_reader<C>(mut self, key: SyphonFormat, constructor: C) -> Self
    where
        C: Fn(Box<dyn Read>) -> Result<Box<dyn FormatReader>, SyphonError> + 'static,
    {
        let format = self.formats.entry(key).or_insert((None, None, None, None));
        format.2 = Some(Box::new(constructor));

        self
    }

    pub fn register_writer<C>(mut self, key: SyphonFormat, constructor: C) -> Self
    where
        C: Fn(Box<dyn Write>, FormatData) -> Result<Box<dyn FormatWriter>, SyphonError> + 'static,
    {
        let format = self.formats.entry(key).or_insert((None, None, None, None));
        format.3 = Some(Box::new(constructor));

        self
    }

    pub fn register_format<F, R, W>(
        mut self,
        key: SyphonFormat,
        identifiers: &'static FormatIdentifiers,
        data_filler: F,
        reader_constructor: R,
        writer_constructor: W,
    ) -> Self
    where
        F: Fn(&mut FormatData) -> Result<(), SyphonError> + 'static,
        R: Fn(Box<dyn Read>) -> Result<Box<dyn FormatReader>, SyphonError> + 'static,
        W: Fn(Box<dyn Write>, FormatData) -> Result<Box<dyn FormatWriter>, SyphonError> + 'static,
    {
        self.formats.insert(
            key,
            (
                Some(identifiers),
                Some(Box::new(data_filler)),
                Some(Box::new(reader_constructor)),
                Some(Box::new(writer_constructor)),
            ),
        );

        self
    }

    pub fn fill_data(&self, key: &SyphonFormat, data: &mut FormatData) -> Result<(), SyphonError> {
        let filler = self.formats.get(key).and_then(|f| f.1.as_ref());

        if let Some(filler) = filler {
            filler(data)?;
        }

        Ok(())
    }

    pub fn construct_reader(
        &self,
        key: &SyphonFormat,
        source: Box<dyn Read>,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        let constructor = self
            .formats
            .get(key)
            .and_then(|f| f.2.as_ref())
            .ok_or(SyphonError::Unsupported)?;

        constructor(source)
    }

    pub fn construct_writer(
        &self,
        key: &SyphonFormat,
        sink: Box<dyn Write>,
        data: FormatData,
    ) -> Result<Box<dyn FormatWriter>, SyphonError> {
        let constructor = self
            .formats
            .get(key)
            .and_then(|f| f.3.as_ref())
            .ok_or(SyphonError::Unsupported)?;

        constructor(sink, data)
    }

    pub fn resolve_format(&self, source: &mut impl Read) -> Result<SyphonFormat, SyphonError> {
        todo!()
    }

    pub fn resolve_reader(
        &self,
        mut source: Box<dyn Read>,
        identifier: Option<FormatIdentifier>,
    ) -> Result<Box<dyn FormatReader>, SyphonError> {
        let key = if let Some(id) = identifier {
            *self
                .formats
                .iter()
                .find(|(_, (ids, _, _, _))| ids.is_some_and(|ids| ids.contains(&id)))
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
        SyphonFormat::Wave,
        &WAVE_IDENTIFIERS,
        fill_wave_data,
        |source| Ok(Box::new(Wave::read(source)?.into_format())),
        |sink, data| {
            Ok(Box::new(
                Wave::write(sink, data.try_into()?)?.into_format(),
            ))
        },
    )
}

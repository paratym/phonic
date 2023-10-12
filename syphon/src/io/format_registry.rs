use crate::io::{
    codec_registry::SyphonCodec,
    formats::{FormatReader, WavReader, WAV_FORMAT_IDENTIFIERS},
};
use std::{collections::HashMap, hash::Hash, io::Read};

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum SyphonFormat {
    Wav,
    Other(&'static str),
}

pub struct FormatRegistry<K, C> {
    formats: HashMap<
        K,
        (
            Option<&'static FormatIdentifiers>,
            Option<Box<dyn Fn(Box<dyn Read>) -> Box<dyn FormatReader<C>>>>, // reader constructor
        ),
    >,
}

pub struct FormatIdentifiers {
    pub file_extensions: &'static [&'static str],
    pub mime_types: &'static [&'static str],
    pub markers: &'static [&'static [u8]],
}

pub struct FormatHint<'a> {
    pub file_extension: Option<&'a str>,
    pub mime_type: Option<&'a str>,
}

impl<K: Hash + Eq + Copy, C> FormatRegistry<K, C> {
    pub fn new() -> Self {
        Self {
            formats: HashMap::new(),
        }
    }

    pub fn register_identifiers(mut self, key: K, identifiers: &'static FormatIdentifiers) -> Self {
        let entry = self.formats.entry(key).or_insert((None, None));
        entry.0 = Some(identifiers);

        self
    }

    pub fn register_reader(
        mut self,
        key: K,
        reader_constructor: impl Fn(Box<dyn Read>) -> Box<dyn FormatReader<C>> + 'static,
    ) -> Self {
        let format = self.formats.entry(key).or_insert((None, None));
        format.1 = Some(Box::new(reader_constructor));

        self
    }

    pub fn register_format<R>(
        mut self,
        key: K,
        identifiers: &'static FormatIdentifiers,
        reader_constructor: R,
    ) -> Self
    where
        R: Fn(Box<dyn Read>) -> Box<dyn FormatReader<C>> + 'static,
    {
        self.formats
            .insert(key, (Some(identifiers), Some(Box::new(reader_constructor))));

        self
    }

    pub fn construct_reader(
        &self,
        key: &K,
        reader: Box<dyn Read>,
    ) -> Option<Box<dyn FormatReader<C>>> {
        Some(self.formats.get(key)?.1.as_ref()?(reader))
    }

    pub fn resolve_format(&self, reader: &mut impl Read, hint: Option<&FormatHint>) -> Option<K> {
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
        
        None
    }

    pub fn resolve_reader(
        &self,
        mut reader: Box<dyn Read>,
        hint: Option<&FormatHint>,
    ) -> Option<Box<dyn FormatReader<C>>> {
        let key = self.resolve_format(&mut reader, hint)?;
        self.construct_reader(&key, reader)
    }
}

impl<'a> FormatHint<'a> {
    pub fn new() -> Self {
        Self {
            file_extension: None,
            mime_type: None,
        }
    }

    pub fn file_extension(mut self, file_extension: &'a str) -> Self {
        self.file_extension = Some(file_extension);
        self
    }

    pub fn mime_type(mut self, mime_type: &'a str) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    pub fn matches(&self, identifiers: &FormatIdentifiers) -> bool {
        if let Some(file_extension) = self.file_extension {
            if identifiers.file_extensions.contains(&file_extension) {
                return true;
            }
        }

        if let Some(mime_type) = self.mime_type {
            if identifiers.mime_types.contains(&mime_type) {
                return true;
            }
        }

        false
    }
}

pub fn syphon_format_registry() -> FormatRegistry<SyphonFormat, SyphonCodec> {
    FormatRegistry::new().register_format(SyphonFormat::Wav, &WAV_FORMAT_IDENTIFIERS, |reader| {
        Box::new(WavReader::new(reader))
    })
}

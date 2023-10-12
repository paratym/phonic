use crate::io::{
    codec_registry::SyphonCodecKey,
    formats::{FormatReader, WavReader},
};
use std::{collections::HashMap, hash::Hash, io::Read};

#[derive(Eq, PartialEq, Hash)]
pub enum SyphonFormatKey {
    Wav,
    Other(&'static str),
}

pub struct FormatReaderRegistry<F, C> {
    reader_constructors: HashMap<F, Box<dyn Fn(Box<dyn Read>) -> Box<dyn FormatReader<C>>>>,
}

impl<F: Hash + Eq, C> FormatReaderRegistry<F, C> {
    pub fn new() -> Self {
        Self {
            reader_constructors: HashMap::new(),
        }
    }

    pub fn reader<T>(mut self, key: F, constructor: T) -> Self
    where
        T: Fn(Box<dyn Read>) -> Box<dyn FormatReader<C>> + 'static,
    {
        self.reader_constructors.insert(key, Box::new(constructor));
        self
    }

    pub fn construct_reader(
        &self,
        key: &F,
        reader: Box<dyn Read>,
    ) -> Option<Box<dyn FormatReader<C>>> {
        Some(self.reader_constructors.get(key)?(reader))
    }
}

pub fn syphon_format_reader_registry() -> FormatReaderRegistry<SyphonFormatKey, SyphonCodecKey> {
    FormatReaderRegistry::new().reader(SyphonFormatKey::Wav, |reader| {
        Box::new(WavReader::new(reader))
    })
}

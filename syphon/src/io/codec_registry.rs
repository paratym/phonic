use crate::{
    core::{SampleReaderRef, SignalSpec},
    io::codecs::PcmDecoder,
};
use std::{collections::HashMap, io::Read, hash::Hash};

#[derive(Eq, PartialEq, Hash)]
pub enum SyphonCodecKey {
    Pcm,
    Other(&'static str),
}

pub struct DecoderRegistry<K> {
    decoder_constructors:
        HashMap<K, Box<dyn Fn(Box<dyn Read>, SignalSpec) -> Option<SampleReaderRef>>>,
}

impl<K: Eq + Hash> DecoderRegistry<K> {
    pub fn new() -> Self {
        Self {
            decoder_constructors: HashMap::new(),
        }
    }

    pub fn decoder<F>(mut self, key: K, constructor: F) -> Self
    where
        F: Fn(Box<dyn Read>, SignalSpec) -> Option<SampleReaderRef> + 'static,
    {
        self.decoder_constructors
            .insert(key, Box::new(constructor));

        self
    }

    pub fn construct_decoder(
        &self,
        key: &K,
        reader: Box<dyn Read>,
        signal_spec: SignalSpec,
    ) -> Option<SampleReaderRef> {
        self.decoder_constructors.get(key)?(reader, signal_spec)
    }
}

pub fn syphon_decoder_registry() -> DecoderRegistry<SyphonCodecKey> {
    DecoderRegistry::new().decoder(SyphonCodecKey::Pcm, |reader, spec| {
        PcmDecoder::new(reader, spec).try_into_sample_reader_ref()
    })
}

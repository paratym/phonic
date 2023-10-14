use crate::io::{codecs::PcmDecoder, SignalSpec, SampleReaderRef};
use std::{collections::HashMap, hash::Hash, io::Read};

use super::SignalReader;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum SyphonCodec {
    Pcm,
    Other(&'static str),
}

pub struct CodecRegistry<K> {
    decoder_constructors:
        HashMap<K, Box<dyn Fn(Box<dyn SignalReader>) -> Option<SampleReaderRef>>>,
}

impl<K: Eq + Hash> CodecRegistry<K> {
    pub fn new() -> Self {
        Self {
            decoder_constructors: HashMap::new(),
        }
    }

    pub fn register_decoder<F>(mut self, key: K, constructor: F) -> Self
    where
        F: Fn(Box<dyn SignalReader>) -> Option<SampleReaderRef> + 'static,
    {
        self.decoder_constructors.insert(key, Box::new(constructor));

        self
    }

    pub fn construct_decoder(
        &self,
        key: &K,
        reader: Box<dyn SignalReader>,
    ) -> Option<SampleReaderRef> {
        self.decoder_constructors.get(key)?(reader)
    }
}

pub fn syphon_codec_registry() -> CodecRegistry<SyphonCodec> {
    CodecRegistry::new().register_decoder(SyphonCodec::Pcm, |reader| {
        PcmDecoder::new(reader).try_into_sample_reader_ref()
    })
}

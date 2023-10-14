use crate::{
    core::{SampleReaderRef, SignalSpec},
    io::codecs::PcmDecoder,
};
use std::{collections::HashMap, hash::Hash, io::Read};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum SyphonCodec {
    Pcm,
    Other(&'static str),
}

pub struct CodecRegistry<K> {
    decoder_constructors:
        HashMap<K, Box<dyn Fn(Box<dyn Read>, SignalSpec) -> Option<SampleReaderRef>>>,
}

impl<K: Eq + Hash> CodecRegistry<K> {
    pub fn new() -> Self {
        Self {
            decoder_constructors: HashMap::new(),
        }
    }

    pub fn register_decoder<F>(mut self, key: K, constructor: F) -> Self
    where
        F: Fn(Box<dyn Read>, SignalSpec) -> Option<SampleReaderRef> + 'static,
    {
        self.decoder_constructors.insert(key, Box::new(constructor));

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

pub fn syphon_codec_registry() -> CodecRegistry<SyphonCodec> {
    CodecRegistry::new().register_decoder(SyphonCodec::Pcm, |reader, spec| {
        PcmDecoder::new(reader, spec).try_into_sample_reader_ref()
    })
}

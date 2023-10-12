use crate::{
    core::{SampleReaderRef, SignalSpec},
    io::codecs::PcmDecoder,
};
use std::{collections::HashMap, io::Read};

pub struct DecoderRegistry {
    decoder_constructors:
        HashMap<&'static str, Box<dyn Fn(Box<dyn Read>, SignalSpec) -> Option<SampleReaderRef>>>,
}

impl DecoderRegistry {
    pub fn new() -> Self {
        Self {
            decoder_constructors: HashMap::new(),
        }
    }

    pub fn decoder<F>(mut self, key: &'static str, constructor: F) -> Self
    where
        F: Fn(Box<dyn Read>, SignalSpec) -> Option<SampleReaderRef> + 'static,
    {
        self.decoder_constructors
            .insert(key, Box::new(constructor));

        self
    }

    pub fn construct_decoder(
        &self,
        key: &str,
        reader: Box<dyn Read>,
        signal_spec: SignalSpec,
    ) -> Option<SampleReaderRef> {
        self.decoder_constructors.get(key)?(reader, signal_spec)
    }
}

pub fn syphon_decoder_registry() -> DecoderRegistry {
    DecoderRegistry::new().decoder("pcm", |reader, spec| {
        PcmDecoder::new(reader, spec).try_into_sample_reader_ref()
    })
}

use crate::io::{codecs::PcmDecoder, SignalSpec, SampleReader};
use std::{collections::HashMap, hash::Hash, io::Read};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum SyphonCodec {
    Pcm,
    Other(&'static str),
}

pub enum SampleReaderRef {
    I8(Box<dyn SampleReader<i8>>),
    I16(Box<dyn SampleReader<i16>>),
    I32(Box<dyn SampleReader<i32>>),
    I64(Box<dyn SampleReader<i64>>),

    U8(Box<dyn SampleReader<u8>>),
    U16(Box<dyn SampleReader<u16>>),
    U32(Box<dyn SampleReader<u32>>),
    U64(Box<dyn SampleReader<u64>>),

    F32(Box<dyn SampleReader<f32>>),
    F64(Box<dyn SampleReader<f64>>),
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

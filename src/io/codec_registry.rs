use crate::{
    io::{codecs::PcmDecoder, EncodedStreamReader, SampleReaderRef},
    SyphonError,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum SyphonCodec {
    Pcm,
    Other(&'static str),
}

pub struct CodecRegistry {
    decoder_constructors: HashMap<
        SyphonCodec,
        Box<dyn Fn(Box<dyn EncodedStreamReader>) -> Result<SampleReaderRef, SyphonError>>,
    >,
}

impl CodecRegistry {
    pub fn new() -> Self {
        Self {
            decoder_constructors: HashMap::new(),
        }
    }

    pub fn register_decoder<F>(mut self, key: SyphonCodec, constructor: F) -> Self
    where
        F: Fn(Box<dyn EncodedStreamReader>) -> Result<SampleReaderRef, SyphonError> + 'static,
    {
        self.decoder_constructors.insert(key, Box::new(constructor));

        self
    }

    pub fn construct_decoder(
        &self,
        reader: impl EncodedStreamReader + 'static,
    ) -> Result<SampleReaderRef, SyphonError> {
        let key = reader.stream_spec().codec_key;
        self.decoder_constructors
            .get(&key)
            .ok_or(SyphonError::Unsupported)?(Box::new(reader))
    }
}

pub fn syphon_codec_registry() -> CodecRegistry {
    CodecRegistry::new().register_decoder(SyphonCodec::Pcm, |reader| {
        Ok(PcmDecoder::new(reader)?.into_sample_reader_ref())
    })
}

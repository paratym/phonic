use crate::{
    io::{codecs::PcmDecoder, MediaStreamReader, SampleReaderRef, SyphonCodec},
    SyphonError,
};
use std::{collections::HashMap, hash::Hash, io::Read};

pub struct CodecRegistry {
    decoder_constructors: HashMap<
        SyphonCodec,
        Box<dyn Fn(Box<dyn MediaStreamReader>) -> Result<SampleReaderRef, SyphonError>>,
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
        F: Fn(Box<dyn MediaStreamReader>) -> Result<SampleReaderRef, SyphonError> + 'static,
    {
        self.decoder_constructors.insert(key, Box::new(constructor));

        self
    }

    pub fn construct_decoder(
        &self,
        reader: impl MediaStreamReader + 'static,
    ) -> Result<SampleReaderRef, SyphonError> {
        let key = reader.stream_spec().codec_key;
        self.decoder_constructors
            .get(&key)
            .ok_or(SyphonError::Unsupported)?(Box::new(reader))
    }
}

pub fn syphon_codec_registry() -> CodecRegistry {
    CodecRegistry::new().register_decoder(SyphonCodec::Pcm, |reader| {
        PcmDecoder::new(reader).try_into_sample_reader_ref()
    })
}

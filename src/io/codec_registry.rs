use crate::{
    io::{
        codecs::*, EncodedStream, EncodedStreamReader, EncodedStreamWriter, SampleReaderRef,
        SampleWriterRef,
    },
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
        (
            Option<
                Box<dyn Fn(Box<dyn EncodedStreamReader>) -> Result<SampleReaderRef, SyphonError>>,
            >,
            Option<
                Box<dyn Fn(Box<dyn EncodedStreamWriter>) -> Result<SampleWriterRef, SyphonError>>,
            >,
        ),
    >,
}

impl CodecRegistry {
    pub fn new() -> Self {
        Self {
            decoder_constructors: HashMap::new(),
        }
    }

    pub fn register_decoder<C>(mut self, key: SyphonCodec, constructor: C) -> Self
    where
        C: Fn(Box<dyn EncodedStreamReader>) -> Result<SampleReaderRef, SyphonError> + 'static,
    {
        let codec = self.decoder_constructors.entry(key).or_insert((None, None));
        codec.0 = Some(Box::new(constructor));

        self
    }

    pub fn register_encoder<C>(mut self, key: SyphonCodec, constructor: C) -> Self
    where
        C: Fn(Box<dyn EncodedStreamWriter>) -> Result<SampleWriterRef, SyphonError> + 'static,
    {
        let codec = self.decoder_constructors.entry(key).or_insert((None, None));
        codec.1 = Some(Box::new(constructor));

        self
    }

    pub fn register_codec<D, E>(
        mut self,
        key: SyphonCodec,
        decoder_constructor: D,
        encoder_constructor: E,
    ) -> Self
    where
        D: Fn(Box<dyn EncodedStreamReader>) -> Result<SampleReaderRef, SyphonError> + 'static,
        E: Fn(Box<dyn EncodedStreamWriter>) -> Result<SampleWriterRef, SyphonError> + 'static,
    {
        self.decoder_constructors.insert(
            key,
            (
                Some(Box::new(decoder_constructor)),
                Some(Box::new(encoder_constructor)),
            ),
        );

        self
    }

    pub fn construct_decoder(
        &self,
        reader: Box<dyn EncodedStreamReader>,
    ) -> Result<SampleReaderRef, SyphonError> {
        let constructor = self
            .decoder_constructors
            .get(&reader.spec().codec_key)
            .map(|c| c.0.as_ref())
            .flatten()
            .ok_or(SyphonError::Unsupported)?;

        constructor(reader)
    }

    pub fn construct_encoder(
        &self,
        writer: Box<dyn EncodedStreamWriter>,
    ) -> Result<SampleWriterRef, SyphonError> {
        let constructor = self
            .decoder_constructors
            .get(&writer.spec().codec_key)
            .map(|c| c.1.as_ref())
            .flatten()
            .ok_or(SyphonError::Unsupported)?;

        constructor(writer)
    }
}

pub fn syphon_codec_registry() -> CodecRegistry {
    CodecRegistry::new().register_codec(
        SyphonCodec::Pcm,
        |reader| Ok(PcmCodec::from_stream(reader)?.into()),
        |writer| Ok(PcmCodec::from_stream(writer)?.into()),
    )
}

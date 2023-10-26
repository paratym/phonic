use crate::{
    io::{
        codecs::pcm::{fill_pcm_spec, PcmCodec},
        EncodedStream, EncodedStreamReader, EncodedStreamSpecBuilder, EncodedStreamWriter,
        SignalReaderRef, SignalWriterRef,
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
    codecs: HashMap<
        SyphonCodec,
        (
            Option<Box<dyn Fn(&mut EncodedStreamSpecBuilder) -> Result<(), SyphonError>>>,
            Option<
                Box<dyn Fn(Box<dyn EncodedStreamReader>) -> Result<SignalReaderRef, SyphonError>>,
            >,
            Option<
                Box<dyn Fn(Box<dyn EncodedStreamWriter>) -> Result<SignalWriterRef, SyphonError>>,
            >,
        ),
    >,
}

impl CodecRegistry {
    pub fn new() -> Self {
        Self {
            codecs: HashMap::new(),
        }
    }

    pub fn register_spec_filler<F>(&mut self, key: SyphonCodec, filler: F)
    where
        F: Fn(&mut EncodedStreamSpecBuilder) -> Result<(), SyphonError> + 'static,
    {
        let codec = self.codecs.entry(key).or_insert((None, None, None));
        codec.0 = Some(Box::new(filler));
    }

    pub fn register_decoder<C>(&mut self, key: SyphonCodec, constructor: C)
    where
        C: Fn(Box<dyn EncodedStreamReader>) -> Result<SignalReaderRef, SyphonError> + 'static,
    {
        let codec = self.codecs.entry(key).or_insert((None, None, None));
        codec.1 = Some(Box::new(constructor));
    }

    pub fn register_encoder<C>(&mut self, key: SyphonCodec, constructor: C)
    where
        C: Fn(Box<dyn EncodedStreamWriter>) -> Result<SignalWriterRef, SyphonError> + 'static,
    {
        let codec = self.codecs.entry(key).or_insert((None, None, None));
        codec.2 = Some(Box::new(constructor));
    }

    pub fn register_codec<F, D, E>(
        mut self,
        key: SyphonCodec,
        spec_filler: F,
        decoder_constructor: D,
        encoder_constructor: E,
    ) -> Self
    where
        F: Fn(&mut EncodedStreamSpecBuilder) -> Result<(), SyphonError> + 'static,
        D: Fn(Box<dyn EncodedStreamReader>) -> Result<SignalReaderRef, SyphonError> + 'static,
        E: Fn(Box<dyn EncodedStreamWriter>) -> Result<SignalWriterRef, SyphonError> + 'static,
    {
        self.codecs.insert(
            key,
            (
                Some(Box::new(spec_filler)),
                Some(Box::new(decoder_constructor)),
                Some(Box::new(encoder_constructor)),
            ),
        );

        self
    }

    pub fn fill_spec(&self, spec: &mut EncodedStreamSpecBuilder) -> Result<(), SyphonError> {
        if let Some(key) = spec.codec_key {
            let filler = self
                .codecs
                .get(&key)
                .map(|c| c.0.as_ref())
                .flatten()
                .ok_or(SyphonError::Unsupported)?;

            filler(spec)?;
        }

        Ok(())
    }

    pub fn construct_decoder(
        &self,
        reader: Box<dyn EncodedStreamReader>,
    ) -> Result<SignalReaderRef, SyphonError> {
        let constructor = reader
            .spec()
            .codec_key
            .as_ref()
            .and_then(|key| self.codecs.get(key).and_then(|c| c.1.as_ref()))
            .ok_or(SyphonError::Unsupported)?;

        constructor(reader)
    }

    pub fn construct_encoder(
        &self,
        writer: Box<dyn EncodedStreamWriter>,
    ) -> Result<SignalWriterRef, SyphonError> {
        let constructor = writer
            .spec()
            .codec_key
            .as_ref()
            .and_then(|key| self.codecs.get(key).and_then(|c| c.2.as_ref()))
            .ok_or(SyphonError::Unsupported)?;

        constructor(writer)
    }
}

pub fn syphon_codec_registry() -> CodecRegistry {
    CodecRegistry::new().register_codec(
        SyphonCodec::Pcm,
        fill_pcm_spec,
        |reader| Ok(PcmCodec::from_stream(reader)?.into()),
        |writer| Ok(PcmCodec::from_stream(writer)?.into()),
    )
}

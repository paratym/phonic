use crate::{
    io::{KnownSampleType, SyphonCodec, TaggedSignalReader, TaggedSignalWriter},
    signal::{Sample, Signal, SignalSpecBuilder},
    SyphonError,
};
use std::{
    any::TypeId,
    io::{Read, Write},
};

#[derive(Debug, Clone, Copy)]
pub struct StreamSpec {
    pub codec: Option<SyphonCodec>,
    pub compression_ratio: Option<f64>,
    pub sample_type: Option<TypeId>,
    pub decoded_spec: SignalSpecBuilder,
}

impl StreamSpec {
    pub fn new() -> Self {
        Self {
            codec: None,
            compression_ratio: None,
            sample_type: None,
            decoded_spec: SignalSpecBuilder::new(),
        }
    }

    pub fn byte_len(&self) -> Option<u64> {
        let sample_type = KnownSampleType::try_from(self.sample_type?).ok()?;
        let decoded_byte_len = self.decoded_spec.n_frames? * sample_type.byte_size() as u64;
        Some((decoded_byte_len as f64 * self.compression_ratio?) as u64)
    }

    pub fn with_codec(mut self, codec: SyphonCodec) -> Self {
        self.codec = Some(codec);
        self
    }

    pub fn with_compression_ratio(mut self, ratio: f64) -> Self {
        self.compression_ratio = Some(ratio);
        self
    }

    pub fn with_sample_type_id(mut self, sample_type: TypeId) -> Self {
        self.sample_type = Some(sample_type);
        self
    }

    pub fn with_sample_type<T: Sample + 'static>(mut self) -> Self {
        self.sample_type = Some(TypeId::of::<T>());
        self
    }

    pub fn with_decoded_spec(mut self, decoded_spec: SignalSpecBuilder) -> Self {
        self.decoded_spec = decoded_spec;
        self
    }

    pub fn filled(self) -> Result<Self, SyphonError> {
        SyphonCodec::fill_spec(self)
    }
}

impl<T: Signal> From<&T> for StreamSpec
where
    T::Sample: 'static,
{
    fn from(inner: &T) -> Self {
        Self {
            codec: None,
            compression_ratio: None,
            sample_type: Some(TypeId::of::<T::Sample>()),
            decoded_spec: inner.spec().clone().into(),
        }
    }
}

pub trait Stream {
    fn spec(&self) -> &StreamSpec;
}

pub trait StreamReader: Stream + Read {
    fn into_decoder(self) -> Result<TaggedSignalReader, SyphonError>
    where
        Self: Sized + 'static,
    {
        SyphonCodec::construct_decoder_reader(self)
    }
}

impl<T> StreamReader for T where T: Stream + Read {}

pub trait StreamWriter: Stream + Write {
    fn into_encoder(self) -> Result<TaggedSignalWriter, SyphonError>
    where
        Self: Sized + 'static,
    {
        SyphonCodec::construct_encoder_writer(self)
    }
}

impl<T> StreamWriter for T where T: Stream + Write {}

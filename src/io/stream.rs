use crate::{
    io::{codecs::KnownCodec, KnownSampleType, TaggedSignalReader, TaggedSignalWriter},
    signal::{Sample, Signal, SignalSpecBuilder},
    SyphonError,
};
use std::{
    any::TypeId,
    io::{Read, Write},
};

#[derive(Debug, Clone, Copy)]
pub struct StreamSpec {
    pub avg_bitrate: Option<f64>,
    pub sample_type: Option<TypeId>,
    pub decoded_spec: SignalSpecBuilder,
}

impl StreamSpec {
    pub fn new() -> Self {
        Self {
            avg_bitrate: None,
            sample_type: None,
            decoded_spec: SignalSpecBuilder::new(),
        }
    }

    pub fn with_avg_bitrate(mut self, bitrate: f64) -> Self {
        self.avg_bitrate = Some(bitrate);
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

    pub fn n_bytes(&self) -> Option<u64> {
        self.avg_bitrate
            .zip(self.decoded_spec.duration())
            .map(|(r, d)| (r / 8.0 * d.as_secs_f64()) as u64)
    }
}

impl<T> From<&T> for StreamSpec
where
    T: Signal,
    T::Sample: 'static,
{
    fn from(inner: &T) -> Self {
        Self {
            avg_bitrate: None,
            sample_type: Some(TypeId::of::<T::Sample>()),
            decoded_spec: inner.spec().clone().into(),
        }
    }
}

pub trait Stream {
    type Codec: KnownCodec;

    fn codec(&self) -> Option<&Self::Codec>;
    fn spec(&self) -> &StreamSpec;
}

pub trait StreamReader: Stream + Read {
    fn into_decoder(self) -> Result<TaggedSignalReader, SyphonError>
    where
        Self: Sized + 'static,
    {
        Self::Codec::decoder_reader(self)
    }
}

impl<T> StreamReader for T where T: Stream + Read {}

pub trait StreamWriter: Stream + Write {
    fn into_encoder(self) -> Result<TaggedSignalWriter, SyphonError>
    where
        Self: Sized + 'static,
    {
        Self::Codec::encoder_writer(self)
    }
}

impl<T> StreamWriter for T where T: Stream + Write {}

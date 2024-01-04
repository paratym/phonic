use crate::{
    io::{SyphonCodec, TaggedSignalReader, TaggedSignalWriter},
    Channels, SampleType, SignalSpecBuilder, SyphonError,
};
use std::{
    io::{Read, Write},
    ops::Deref,
};

#[derive(Debug, Clone, Copy)]
pub struct StreamSpec {
    pub codec: Option<SyphonCodec>,
    pub byte_len: Option<u64>,
    pub sample_type: Option<SampleType>,
    pub decoded_spec: SignalSpecBuilder,
}

impl StreamSpec {
    pub fn new() -> Self {
        Self {
            codec: None,
            byte_len: None,
            sample_type: None,
            decoded_spec: SignalSpecBuilder::new(),
        }
    }

    pub fn with_codec(mut self, codec: SyphonCodec) -> Self {
        self.codec = Some(codec);
        self
    }

    pub fn with_byte_len(mut self, byte_len: Option<u64>) -> Self {
        self.byte_len = byte_len;
        self
    }

    pub fn with_sample_type(mut self, sample_type: SampleType) -> Self {
        self.sample_type = Some(sample_type);
        self
    }

    pub fn with_decoded_spec(mut self, decoded_spec: SignalSpecBuilder) -> Self {
        self.decoded_spec = decoded_spec;
        self
    }

    pub fn compression_ratio(&self) -> Option<f64> {
        let decoded_byte_len = self
            .sample_type
            .zip(self.decoded_spec.n_samples())
            .map(|(s, n)| s.byte_size() as u64 * n);

        self.byte_len
            .zip(decoded_byte_len)
            .map(|(encoded, decoded)| decoded as f64 / encoded as f64)
    }

    pub fn set_compression_ratio(&mut self, ratio: f64) -> Result<(), SyphonError> {
        let bytes_per_encoded_sample = self.sample_type.map(|s| s.byte_size() as f64 * ratio);

        let calculated_byte_len = self
            .decoded_spec
            .n_samples()
            .zip(bytes_per_encoded_sample)
            .map(|(n_samples, bytes_per_sample)| (n_samples as f64 * bytes_per_sample) as u64);

        if calculated_byte_len.is_some_and(|n| self.byte_len.get_or_insert(n) != &n) {
            return Err(SyphonError::InvalidData);
        }

        let calculated_n_frames = self
            .byte_len
            .zip(bytes_per_encoded_sample)
            .zip(self.decoded_spec.channels)
            .map(|((byte_len, bytes_per_sample), channels)| {
                ((byte_len as f64 / bytes_per_sample) / channels.count() as f64) as u64
            });

        if calculated_n_frames.is_some_and(|n| self.decoded_spec.n_frames.get_or_insert(n) != &n) {
            return Err(SyphonError::InvalidData);
        }

        let calculated_channels = self
            .byte_len
            .zip(bytes_per_encoded_sample)
            .zip(self.decoded_spec.n_frames)
            .map(|((byte_len, bytes_per_sample), n_frames)| {
                ((byte_len as f64 / bytes_per_sample) / n_frames as f64) as u32
            });

        if calculated_channels
            .is_some_and(|c| self.decoded_spec.channels.get_or_insert(c.into()).count() != c)
        {
            return Err(SyphonError::InvalidData);
        }

        Ok(())
    }

    pub fn filled(mut self) -> Result<Self, SyphonError> {
        SyphonCodec::fill_spec(&mut self)?;
        Ok(self)
    }
}

pub trait Stream {
    fn spec(&self) -> &StreamSpec;

    fn into_encoder(self) -> Result<TaggedSignalWriter, SyphonError>
    where
        Self: Sized + Write + 'static,
    {
        let spec = *self.spec();
        spec.codec
            .ok_or(SyphonError::MissingData)?
            .construct_encoder(self, spec)
    }

    fn into_decoder(self) -> Result<TaggedSignalReader, SyphonError>
    where
        Self: Sized + Read + 'static,
    {
        let spec = *self.spec();
        spec.codec
            .ok_or(SyphonError::MissingData)?
            .construct_decoder(self, spec)
    }
}

impl<T> Stream for T
where
    T: Deref,
    T::Target: Stream,
{
    fn spec(&self) -> &StreamSpec {
        self.deref().spec()
    }
}

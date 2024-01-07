use crate::{
    io::{SyphonCodec, TaggedSignalReader, TaggedSignalWriter},
    KnownSampleType, Sample, SignalSpec, SignalSpecBuilder, SyphonError,
};
use std::{
    any::TypeId,
    io::{Read, Write},
};

#[derive(Debug, Clone)]
pub struct StreamSpec {
    pub codec: SyphonCodec,
    pub byte_len: Option<u64>,
    pub sample_type: TypeId,
    pub decoded_spec: SignalSpec,
}

#[derive(Debug, Clone)]
pub struct StreamSpecBuilder {
    pub codec: Option<SyphonCodec>,
    pub byte_len: Option<u64>,
    pub sample_type: Option<TypeId>,
    pub decoded_spec: SignalSpecBuilder,
}

impl StreamSpec {
    pub fn builder() -> StreamSpecBuilder {
        StreamSpecBuilder::new()
    }

    pub fn known_sample_type(&self) -> Option<KnownSampleType> {
        self.sample_type.try_into().ok()
    }

    pub fn compression_ratio(&self) -> Option<f64> {
        self.byte_len
            .zip(self.decoded_spec.n_samples())
            .zip(self.known_sample_type().map(KnownSampleType::byte_size))
            .map(|((byte_len, n_samples), bytes_per_sample)| {
                (byte_len as f64 / n_samples as f64) / bytes_per_sample as f64
            })
    }
}

impl TryFrom<StreamSpecBuilder> for StreamSpec {
    type Error = SyphonError;

    fn try_from(builder: StreamSpecBuilder) -> Result<Self, Self::Error> {
        Ok(Self {
            codec: builder.codec.ok_or(SyphonError::MissingData)?,
            byte_len: builder.byte_len,
            sample_type: builder.sample_type.ok_or(SyphonError::MissingData)?,
            decoded_spec: builder.decoded_spec.build()?,
        })
    }
}

impl StreamSpecBuilder {
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

    pub fn with_sample_type<T: Sample + 'static>(mut self) -> Self {
        self.sample_type = Some(TypeId::of::<T>());
        self
    }

    pub fn with_decoded_spec(mut self, decoded_spec: SignalSpecBuilder) -> Self {
        self.decoded_spec = decoded_spec;
        self
    }

    pub fn with_compression_ratio(mut self, ratio: f64) -> Result<Self, SyphonError> {
        let bytes_per_encoded_sample = self
            .sample_type
            .and_then(|s| KnownSampleType::try_from(s).ok())
            .map(|s| s.byte_size() as f64 * ratio);

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

        Ok(self)
    }

    pub fn filled(mut self) -> Result<Self, SyphonError> {
        SyphonCodec::fill_spec(self)
    }

    pub fn build(self) -> Result<StreamSpec, SyphonError> {
        self.try_into()
    }
}

impl From<StreamSpec> for StreamSpecBuilder {
    fn from(spec: StreamSpec) -> Self {
        Self {
            codec: Some(spec.codec),
            byte_len: spec.byte_len,
            sample_type: Some(spec.sample_type),
            decoded_spec: spec.decoded_spec.into(),
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
        let spec = self.spec().clone().into();
        SyphonCodec::construct_decoder_reader(self, spec)
    }
}

impl<T> StreamReader for T where T: Stream + Read {}

pub trait StreamWriter: Stream + Write {
    fn into_encoder(self) -> Result<TaggedSignalWriter, SyphonError>
    where
        Self: Sized + 'static,
    {
        let spec = self.spec().clone().into();
        SyphonCodec::construct_encoder_writer(self, spec)
    }
}

impl<T> StreamWriter for T where T: Stream + Write {}

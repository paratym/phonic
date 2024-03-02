use crate::{CodecRegistry, CodecTag};
use std::{
    any::TypeId,
    io::{Read, Write},
};
use syphon_core::SyphonError;
use syphon_signal::{Sample, Signal, SignalSpecBuilder, TaggedSignalReader, TaggedSignalWriter};

#[derive(Debug, Clone, Copy)]
pub struct StreamSpec<C: CodecTag> {
    pub codec: Option<C>,
    pub avg_bitrate: Option<f64>,
    pub block_align: Option<u16>,
    pub sample_type: Option<TypeId>,
    pub decoded_spec: SignalSpecBuilder,
}

impl<C: CodecTag> StreamSpec<C> {
    pub fn new() -> Self {
        Self {
            codec: None,
            avg_bitrate: None,
            block_align: None,
            sample_type: None,
            decoded_spec: SignalSpecBuilder::new(),
        }
    }

    pub fn with_tag_type<T>(mut self) -> StreamSpec<T>
    where
        T: CodecTag,
        C: TryInto<T>,
    {
        StreamSpec {
            codec: self.codec.and_then(|c| c.try_into().ok()),
            avg_bitrate: self.avg_bitrate,
            block_align: self.block_align,
            sample_type: self.sample_type,
            decoded_spec: self.decoded_spec,
        }
    }

    pub fn with_codec(mut self, codec: C) -> Self {
        self.codec = Some(codec);
        self
    }

    pub fn with_avg_bitrate(mut self, bitrate: f64) -> Self {
        self.avg_bitrate = Some(bitrate);
        self
    }

    pub fn with_block_align(mut self, block_align: u16) -> Self {
        self.block_align = Some(block_align);
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

    pub fn is_empty(&self) -> bool {
        self.avg_bitrate.is_none()
            && self.block_align.is_none()
            && self.sample_type.is_none()
            && self.decoded_spec.is_empty()
    }

    pub fn merge(&mut self, other: Self) -> Result<(), SyphonError> {
        if let Some(codec) = other.codec {
            if self.codec.get_or_insert(codec) != &codec {
                return Err(SyphonError::SignalMismatch);
            }
        }

        if let Some(avg_bitrate) = other.avg_bitrate {
            if self.avg_bitrate.get_or_insert(avg_bitrate) != &avg_bitrate {
                return Err(SyphonError::SignalMismatch);
            }
        }

        if let Some(block_align) = other.block_align {
            if self
                .block_align
                .is_some_and(|align| align % block_align != 0)
            {
                return Err(SyphonError::SignalMismatch);
            }

            self.block_align = Some(block_align);
        }

        self.decoded_spec.merge(other.decoded_spec)
    }

    pub fn fill(&mut self) -> Result<(), SyphonError>
    where
        C: CodecRegistry,
    {
        C::fill_spec(self)
    }

    pub fn filled(mut self) -> Result<Self, SyphonError>
    where
        C: CodecRegistry,
    {
        self.fill()?;
        Ok(self)
    }
}

impl<T, C> From<&T> for StreamSpec<C>
where
    T: Signal,
    T::Sample: 'static,
    C: CodecTag,
{
    fn from(inner: &T) -> Self {
        Self {
            codec: None,
            avg_bitrate: None,
            block_align: None,
            sample_type: Some(TypeId::of::<T::Sample>()),
            decoded_spec: inner.spec().clone().into(),
        }
    }
}

pub trait Stream {
    type Tag: CodecTag;

    fn spec(&self) -> &StreamSpec<Self::Tag>;
}

pub trait StreamReader: Stream + Read {
    fn into_decoder(self) -> Result<TaggedSignalReader, SyphonError>
    where
        Self: Sized + 'static,
        Self::Tag: CodecRegistry,
    {
        Self::Tag::decoder_reader(self)
    }
}

impl<T> StreamReader for T where T: Stream + Read {}

pub trait StreamWriter: Stream + Write {
    fn into_encoder(self) -> Result<TaggedSignalWriter, SyphonError>
    where
        Self: Sized + 'static,
        Self::Tag: CodecRegistry,
    {
        Self::Tag::encoder_writer(self)
    }
}

impl<T> StreamWriter for T where T: Stream + Write {}

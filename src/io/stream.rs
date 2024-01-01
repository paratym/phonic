use crate::{
    io::{SyphonCodec, TaggedSignalReader, TaggedSignalWriter},
    SampleType, SignalSpec, SignalSpecBuilder, SyphonError,
};
use std::{
    io::{Read, Write},
    ops::{Deref, DerefMut},
};

#[derive(Clone, Copy, Debug)]
pub struct StreamSpec {
    pub codec: SyphonCodec,
    pub byte_len: Option<u64>,
    pub decoded_spec: SignalSpec,
}

#[derive(Debug, Clone, Copy)]
pub struct StreamSpecBuilder {
    pub codec: Option<SyphonCodec>,
    pub byte_len: Option<u64>,
    pub decoded_spec: SignalSpecBuilder,
}

impl StreamSpec {
    pub fn builder() -> StreamSpecBuilder {
        StreamSpecBuilder::new()
    }
}

impl TryFrom<StreamSpecBuilder> for StreamSpec {
    type Error = SyphonError;

    fn try_from(builder: StreamSpecBuilder) -> Result<Self, Self::Error> {
        Ok(Self {
            codec: builder.codec.unwrap_or(SyphonCodec::Unknown),
            byte_len: builder.byte_len,
            decoded_spec: builder.decoded_spec.build()?,
        })
    }
}

impl StreamSpecBuilder {
    pub fn new() -> Self {
        Self {
            codec: None,
            byte_len: None,
            decoded_spec: SignalSpecBuilder::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.codec.is_none() && self.byte_len.is_none() && self.decoded_spec.is_empty()
    }

    pub fn with_codec(mut self, codec: impl Into<SyphonCodec>) -> Self {
        self.codec = Some(codec.into());
        self
    }

    pub fn with_byte_len(mut self, byte_len: impl Into<Option<u64>>) -> Self {
        self.byte_len = byte_len.into();
        self
    }

    pub fn with_decoded_spec(mut self, decoded_spec: impl Into<SignalSpecBuilder>) -> Self {
        self.decoded_spec = decoded_spec.into();
        self
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
            decoded_spec: spec.decoded_spec.into(),
        }
    }
}

pub trait Stream {
    fn spec(&self) -> &StreamSpec;
}

pub trait StreamReader: Stream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SyphonError>;

    fn into_decoder(self) -> Result<TaggedSignalReader, SyphonError>
    where
        Self: Sized + 'static,
    {
        let codec = self.spec().codec;
        codec.construct_decoder(self)
    }
}

pub trait StreamWriter: Stream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;

    fn into_encoder(self) -> Result<TaggedSignalWriter, SyphonError>
    where
        Self: Sized + 'static,
    {
        let codec = self.spec().codec;
        codec.construct_encoder(self)
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

impl<T> StreamReader for T
where
    T: DerefMut,
    T::Target: StreamReader,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SyphonError> {
        self.deref_mut().read(buf)
    }
}

impl<T> StreamWriter for T
where
    T: DerefMut,
    T::Target: StreamWriter,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, SyphonError> {
        self.deref_mut().write(buf)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.deref_mut().flush()
    }
}

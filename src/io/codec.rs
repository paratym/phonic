use crate::{
    io::{TaggedSignalReader, TaggedSignalWriter, SyphonCodec},
    SampleType, SignalSpec, SignalSpecBuilder, SyphonError,
};
use std::{io::{Read, Write}, ops::{Deref, DerefMut}};

#[derive(Clone, Copy)]
pub struct StreamSpec {
    pub codec: SyphonCodec,
    pub decoded_spec: SignalSpec<SampleType>,
    pub block_size: usize,
    pub byte_len: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub struct StreamSpecBuilder {
    pub codec: Option<SyphonCodec>,
    pub decoded_spec: SignalSpecBuilder<SampleType>,
    pub block_size: Option<usize>,
    pub byte_len: Option<u64>,
}

impl StreamSpec {
    pub fn builder() -> StreamSpecBuilder {
        StreamSpecBuilder::new()
    }

    pub fn n_blocks(&self) -> Option<u64> {
        self.byte_len.map(|n| n / self.block_size as u64)
    }
}

impl TryFrom<StreamSpecBuilder> for StreamSpec {
    type Error = SyphonError;

    fn try_from(builder: StreamSpecBuilder) -> Result<Self, Self::Error> {
        if builder
            .block_size
            .zip(builder.byte_len)
            .map_or(false, |(b, n)| n % b as u64 != 0)
        {
            return Err(SyphonError::InvalidData);
        }

        Ok(Self {
            codec: builder.codec.unwrap_or(SyphonCodec::Unknown),
            decoded_spec: builder.decoded_spec.build()?,
            block_size: builder.block_size.ok_or(SyphonError::InvalidData)?,
            byte_len: builder.byte_len,
        })
    }
}

impl StreamSpecBuilder {
    pub fn new() -> Self {
        Self {
            codec: None,
            decoded_spec: SignalSpecBuilder::new(),
            block_size: None,
            byte_len: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.codec.is_none()
            && self.decoded_spec.is_empty()
            && self.block_size.is_none()
            && self.byte_len.is_none()
    }

    pub fn sample_type(&self) -> Option<SampleType> {
        self.decoded_spec.sample_type()
    }

    pub fn n_blocks(&self) -> Option<u64> {
        self.byte_len
            .zip(self.block_size)
            .map(|(n, b)| n / b as u64)
    }

    pub fn with_codec<T: Into<Option<SyphonCodec>>>(mut self, codec: T) -> Self {
        self.codec = codec.into();
        self
    }

    pub fn with_decoded_spec<T: Into<SignalSpecBuilder<SampleType>>>(
        mut self,
        decoded_spec: T,
    ) -> Self {
        self.decoded_spec = decoded_spec.into();
        self
    }

    pub fn with_block_size<T: Into<Option<usize>>>(mut self, block_size: T) -> Self {
        self.block_size = block_size.into();
        self
    }

    pub fn with_byte_len<T: Into<Option<u64>>>(mut self, byte_len: T) -> Self {
        self.byte_len = byte_len.into();
        self
    }

    pub fn fill(&mut self) -> Result<(), SyphonError> {
        if let Some(codec) = self.codec {
            codec.fill_spec(self)?;
        }

        Ok(())
    }

    pub fn filled(mut self) -> Result<Self, SyphonError> {
        self.fill()?;
        Ok(self)
    }

    pub fn build(self) -> Result<StreamSpec, SyphonError> {
        self.try_into()
    }
}

impl From<StreamSpec> for StreamSpecBuilder {
    fn from(spec: StreamSpec) -> Self {
        Self {
            codec: Some(spec.codec),
            decoded_spec: spec.decoded_spec.into(),
            block_size: Some(spec.block_size),
            byte_len: spec.byte_len,
        }
    }
}

pub trait Stream {
    fn spec(&self) -> &StreamSpec;
}

pub trait StreamReader: Stream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SyphonError>;

    fn as_decoder(&'static mut self) -> Result<TaggedSignalReader, SyphonError>
    where
        Self: Sized,
        &'static mut Self: StreamReader,
    {
        let codec = self.spec().codec;
        codec.construct_decoder(self)
    }

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

    fn as_encoder(&'static mut self) -> Result<TaggedSignalWriter, SyphonError>
    where
        Self: Sized,
        &'static mut Self: StreamWriter,
    {
        let codec = self.spec().codec;
        codec.construct_encoder(self)
    }

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
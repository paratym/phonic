use crate::{
    io::{SyphonCodec, SyphonFormat},
    dsp::adapters::IntoAdapter,
    SignalReader, SignalSpec, SignalSpecBuilder, SignalWriter, SyphonError, FromKnownSample
};
use std::io::{Read, Write};

#[derive(Clone)]
pub struct FormatData {
    pub format_key: Option<SyphonFormat>,
    pub tracks: Vec<EncodedStreamSpecBuilder>,
    // pub metadata: Metadata,
}

impl FormatData {
    pub fn new() -> Self {
        Self {
            format_key: None,
            tracks: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.format_key.is_none() && self.tracks.is_empty()
    }

    pub fn format(mut self, format: SyphonFormat) -> Self {
        self.format_key = Some(format);
        self
    }

    pub fn track(mut self, track: EncodedStreamSpecBuilder) -> Self {
        self.tracks.push(track);
        self
    }
}

pub trait Format {
    fn format_data(&self) -> &FormatData;
}

pub struct FormatReadResult {
    pub track: usize,
    pub n: usize,
}

pub trait FormatReader: Format {
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError>;
}

pub trait FormatWriter: Format {
    fn write(&mut self, track_i: usize, buf: &[u8]) -> Result<usize, SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;
}

impl Format for Box<dyn FormatReader> {
    fn format_data(&self) -> &FormatData {
        self.as_ref().format_data()
    }
}

impl FormatReader for Box<dyn FormatReader> {
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError> {
        self.as_mut().read(buf)
    }
}

impl Format for Box<dyn FormatWriter> {
    fn format_data(&self) -> &FormatData {
        self.as_ref().format_data()
    }
}

impl FormatWriter for Box<dyn FormatWriter> {
    fn write(&mut self, track_i: usize, buf: &[u8]) -> Result<usize, SyphonError> {
        self.as_mut().write(track_i, buf)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.as_mut().flush()
    }
}

#[derive(Clone, Copy)]
pub struct EncodedStreamSpec {
    pub codec_key: Option<SyphonCodec>,
    pub decoded_spec: SignalSpecBuilder,
    pub block_size: usize,
    pub byte_len: Option<u64>,
}

#[derive(Clone, Copy, Default)]
pub struct EncodedStreamSpecBuilder {
    pub codec_key: Option<SyphonCodec>,
    pub decoded_spec: SignalSpecBuilder,
    pub block_size: Option<usize>,
    pub byte_len: Option<u64>,
}

impl EncodedStreamSpec {
    pub fn builder() -> EncodedStreamSpecBuilder {
        EncodedStreamSpecBuilder::new()
    }

    pub fn n_blocks(&self) -> Option<u64> {
        self.byte_len.map(|n| n / self.block_size as u64)
    }
}

impl TryFrom<EncodedStreamSpecBuilder> for EncodedStreamSpec {
    type Error = SyphonError;

    fn try_from(builder: EncodedStreamSpecBuilder) -> Result<Self, Self::Error> {
        if builder
            .block_size
            .zip(builder.byte_len)
            .map_or(false, |(b, n)| n % b as u64 != 0)
        {
            return Err(SyphonError::InvalidData);
        }

        Ok(Self {
            codec_key: builder.codec_key,
            decoded_spec: builder.decoded_spec,
            block_size: builder.block_size.ok_or(SyphonError::InvalidData)?,
            byte_len: builder.byte_len,
        })
    }
}

impl EncodedStreamSpecBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.codec_key.is_none()
            && self.decoded_spec.is_empty()
            && self.block_size.is_none()
            && self.byte_len.is_none()
    }

    pub fn n_blocks(&self) -> Option<u64> {
        self.byte_len
            .zip(self.block_size)
            .map(|(n, b)| n / b as u64)
    }

    pub fn codec(mut self, codec: SyphonCodec) -> Self {
        self.codec_key = Some(codec);
        self
    }

    pub fn decoded_spec(mut self, decoded_spec: SignalSpecBuilder) -> Self {
        self.decoded_spec = decoded_spec;
        self
    }

    pub fn block_size(mut self, block_size: usize) -> Self {
        self.block_size = Some(block_size);
        self
    }

    pub fn byte_len(mut self, byte_len: u64) -> Self {
        self.byte_len = Some(byte_len);
        self
    }

    pub fn build(self) -> Result<EncodedStreamSpec, SyphonError> {
        self.try_into()
    }
}

impl From<EncodedStreamSpec> for EncodedStreamSpecBuilder {
    fn from(spec: EncodedStreamSpec) -> Self {
        Self {
            codec_key: spec.codec_key,
            decoded_spec: spec.decoded_spec,
            block_size: Some(spec.block_size),
            byte_len: spec.byte_len,
        }
    }
}

pub trait EncodedStream {
    fn spec(&self) -> &EncodedStreamSpec;
}

pub trait EncodedStreamReader: EncodedStream + Read {}
pub trait EncodedStreamWriter: EncodedStream + Write {}

impl<T: EncodedStream + Read> EncodedStreamReader for T {}

impl EncodedStream for Box<dyn EncodedStreamReader> {
    fn spec(&self) -> &EncodedStreamSpec {
        self.as_ref().spec()
    }
}

impl<T: EncodedStream + Write> EncodedStreamWriter for T {}

impl EncodedStream for Box<dyn EncodedStreamWriter> {
    fn spec(&self) -> &EncodedStreamSpec {
        self.as_ref().spec()
    }
}

pub enum SignalReaderRef {
    I8(Box<dyn SignalReader<i8>>),
    I16(Box<dyn SignalReader<i16>>),
    I32(Box<dyn SignalReader<i32>>),
    I64(Box<dyn SignalReader<i64>>),

    U8(Box<dyn SignalReader<u8>>),
    U16(Box<dyn SignalReader<u16>>),
    U32(Box<dyn SignalReader<u32>>),
    U64(Box<dyn SignalReader<u64>>),

    F32(Box<dyn SignalReader<f32>>),
    F64(Box<dyn SignalReader<f64>>),
}

pub enum SignalWriterRef {
    I8(Box<dyn SignalWriter<i8>>),
    I16(Box<dyn SignalWriter<i16>>),
    I32(Box<dyn SignalWriter<i32>>),
    I64(Box<dyn SignalWriter<i64>>),

    U8(Box<dyn SignalWriter<u8>>),
    U16(Box<dyn SignalWriter<u16>>),
    U32(Box<dyn SignalWriter<u32>>),
    U64(Box<dyn SignalWriter<u64>>),

    F32(Box<dyn SignalWriter<f32>>),
    F64(Box<dyn SignalWriter<f64>>),
}

macro_rules! match_signal_ref {
    ($ref:ident, $self:ident, $inner:ident, $rhs:expr) => {
        match $ref {
            $self::I8($inner) => $rhs,
            $self::I16($inner) => $rhs,
            $self::I32($inner) => $rhs,
            $self::I64($inner) => $rhs,
            $self::U8($inner) => $rhs,
            $self::U16($inner) => $rhs,
            $self::U32($inner) => $rhs,
            $self::U64($inner) => $rhs,
            $self::F32($inner) => $rhs,
            $self::F64($inner) => $rhs,
        }
    };
}

macro_rules! impl_signal_ref {
    ($self:ty) => {
        impl $self {
            pub fn spec(&self) -> &SignalSpec {
                match_signal_ref!(self, Self, signal, signal.spec())
            }

            pub fn adapt_sample_type<O: FromKnownSample + 'static>(self) -> Box<dyn SignalReader<O>> {
                match_signal_ref!(self, Self, signal, Box::new(signal.adapt_sample_type::<O>()))
            }
        }
    };
}

impl_signal_ref!(SignalReaderRef);
// impl_signal_ref!(SignalWriterRef);
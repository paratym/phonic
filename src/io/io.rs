use crate::{
    io::SyphonCodec, SampleReader, SampleWriter, StreamSpec, StreamSpecBuilder, SyphonError,
};
use std::io::{Read, Write};

#[derive(Clone)]
pub struct FormatData {
    pub tracks: Vec<EncodedStreamSpecBuilder>,
    // pub metadata: Metadata,
}

impl FormatData {
    pub fn new() -> Self {
        Self { tracks: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
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
    pub codec_key: SyphonCodec,
    pub decoded_spec: StreamSpecBuilder,
    pub block_size: usize,
    pub byte_len: Option<u64>,
}

#[derive(Clone, Copy, Default)]
pub struct EncodedStreamSpecBuilder {
    pub codec_key: Option<SyphonCodec>,
    pub decoded_spec: StreamSpecBuilder,
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
            codec_key: builder.codec_key.ok_or(SyphonError::InvalidData)?,
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

    pub fn decoded_spec(mut self, decoded_spec: StreamSpecBuilder) -> Self {
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
            codec_key: Some(spec.codec_key),
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

pub enum SampleReaderRef {
    I8(Box<dyn SampleReader<i8>>),
    I16(Box<dyn SampleReader<i16>>),
    I32(Box<dyn SampleReader<i32>>),
    I64(Box<dyn SampleReader<i64>>),

    U8(Box<dyn SampleReader<u8>>),
    U16(Box<dyn SampleReader<u16>>),
    U32(Box<dyn SampleReader<u32>>),
    U64(Box<dyn SampleReader<u64>>),

    F32(Box<dyn SampleReader<f32>>),
    F64(Box<dyn SampleReader<f64>>),
}

pub enum SampleWriterRef {
    I8(Box<dyn SampleWriter<i8>>),
    I16(Box<dyn SampleWriter<i16>>),
    I32(Box<dyn SampleWriter<i32>>),
    I64(Box<dyn SampleWriter<i64>>),

    U8(Box<dyn SampleWriter<u8>>),
    U16(Box<dyn SampleWriter<u16>>),
    U32(Box<dyn SampleWriter<u32>>),
    U64(Box<dyn SampleWriter<u64>>),

    F32(Box<dyn SampleWriter<f32>>),
    F64(Box<dyn SampleWriter<f64>>),
}

impl SampleReaderRef {
    pub fn spec(&self) -> &StreamSpec {
        match self {
            Self::I8(reader) => reader.spec(),
            Self::I16(reader) => reader.spec(),
            Self::I32(reader) => reader.spec(),
            Self::I64(reader) => reader.spec(),

            Self::U8(reader) => reader.spec(),
            Self::U16(reader) => reader.spec(),
            Self::U32(reader) => reader.spec(),
            Self::U64(reader) => reader.spec(),

            Self::F32(reader) => reader.spec(),
            Self::F64(reader) => reader.spec(),
        }
    }
}

macro_rules! impl_sample_reader_into_ref {
    ($s:ty, $f:ident) => {
        impl From<Box<dyn SampleReader<$s>>> for SampleReaderRef {
            fn from(reader: Box<dyn SampleReader<$s>>) -> Self {
                Self::$f(Box::new(reader))
            }
        }
    };
}

impl_sample_reader_into_ref!(i8, I8);
impl_sample_reader_into_ref!(i16, I16);
impl_sample_reader_into_ref!(i32, I32);
impl_sample_reader_into_ref!(i64, I64);

impl_sample_reader_into_ref!(u8, U8);
impl_sample_reader_into_ref!(u16, U16);
impl_sample_reader_into_ref!(u32, U32);
impl_sample_reader_into_ref!(u64, U64);

impl_sample_reader_into_ref!(f32, F32);
impl_sample_reader_into_ref!(f64, F64);

impl SampleWriterRef {
    pub fn spec(&self) -> &StreamSpec {
        match self {
            Self::I8(writer) => writer.spec(),
            Self::I16(writer) => writer.spec(),
            Self::I32(writer) => writer.spec(),
            Self::I64(writer) => writer.spec(),

            Self::U8(writer) => writer.spec(),
            Self::U16(writer) => writer.spec(),
            Self::U32(writer) => writer.spec(),
            Self::U64(writer) => writer.spec(),

            Self::F32(writer) => writer.spec(),
            Self::F64(writer) => writer.spec(),
        }
    }
}

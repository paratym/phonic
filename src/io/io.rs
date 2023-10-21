use crate::{io::SyphonCodec, Sample, SampleFormat, SyphonError};
use std::io::{Read, Seek, SeekFrom, Write};

pub trait MediaSource: Read + Seek {}

pub trait MediaSink: Write + Seek {}

impl<T: Read + Seek> MediaSource for T {}

impl<T: Write + Seek> MediaSink for T {}

#[derive(Clone)]
pub struct FormatData {
    pub tracks: Vec<EncodedStreamSpecBuilder>,
    // pub metadata: Metadata,
}

pub struct FormatReadResult {
    pub track_i: usize,
    pub n: usize,
}

pub trait FormatReader: Seek {
    fn format_data(&self) -> &FormatData;
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError>;
}

pub trait FormatWriter: Seek {
    fn format_data(&self) -> &FormatData;
    fn write(&mut self, track_i: usize, buf: &[u8]) -> Result<usize, SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;
}

impl FormatReader for Box<dyn FormatReader> {
    fn format_data(&self) -> &FormatData {
        self.as_ref().format_data()
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError> {
        self.as_mut().read(buf)
    }
}

impl FormatWriter for Box<dyn FormatWriter> {
    fn format_data(&self) -> &FormatData {
        self.as_ref().format_data()
    }

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

pub trait EncodedStreamReader: Read + Seek {
    fn stream_spec(&self) -> &EncodedStreamSpec;
}

pub trait EncodedStreamWriter: Write + Seek {
    fn stream_spec(&self) -> &EncodedStreamSpec;
}

impl EncodedStreamReader for Box<dyn EncodedStreamReader> {
    fn stream_spec(&self) -> &EncodedStreamSpec {
        self.as_ref().stream_spec()
    }
}

impl EncodedStreamWriter for Box<dyn EncodedStreamWriter> {
    fn stream_spec(&self) -> &EncodedStreamSpec {
        self.as_ref().stream_spec()
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

    pub fn try_build(self) -> Result<EncodedStreamSpec, SyphonError> {
        Ok(EncodedStreamSpec {
            codec_key: self.codec_key.ok_or(SyphonError::MalformedData)?,
            decoded_spec: self.decoded_spec,
            block_size: self.block_size.ok_or(SyphonError::MalformedData)?,
            byte_len: self.byte_len,
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct StreamSpec {
    pub sample_format: SampleFormat,
    pub n_channels: u8,
    pub sample_rate: u32,
    pub block_size: usize,
    pub n_frames: Option<u64>,
}

#[derive(Default, Clone, Copy)]
pub struct StreamSpecBuilder {
    pub sample_format: Option<SampleFormat>,
    pub n_channels: Option<u8>,
    pub sample_rate: Option<u32>,
    pub block_size: Option<usize>,
    pub n_frames: Option<u64>,
}

pub trait SampleReader<S: Sample> {
    fn stream_spec(&self) -> &StreamSpec;
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;
    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError>;

    // fn into_ref(self: Box<Self>) -> SampleReaderRef {
    //     match S::FORMAT {
    //         SampleFormat::U8 => SampleReaderRef::U8(self),
    //         SampleFormat::U16 => SampleReaderRef::U16(self),
    //         SampleFormat::U32 => SampleReaderRef::U32(self),
    //         SampleFormat::U64 => SampleReaderRef::U64(self),

    //         SampleFormat::I8 => SampleReaderRef::I8(self),
    //         SampleFormat::I16 => SampleReaderRef::I16(self),
    //         SampleFormat::I32 => SampleReaderRef::I32(self),
    //         SampleFormat::I64 => SampleReaderRef::I64(self),

    //         SampleFormat::F32 => SampleReaderRef::F32(self),
    //         SampleFormat::F64 => SampleReaderRef::F64(self),
    //     }
    // }

    fn read_exact(&mut self, mut buffer: &mut [S]) -> Result<(), SyphonError> {
        let block_size = self.stream_spec().block_size;
        if buffer.len() % block_size != 0 {
            return Err(SyphonError::StreamMismatch);
        }

        let mut n_read: usize = 0;
        while !buffer.is_empty() {
            n_read += self.read(buffer)?;
            if n_read == 0 {
                return Err(SyphonError::Empty);
            }

            buffer = &mut buffer[n_read..];
        }

        Ok(())
    }
}

pub trait SampleWriter<S: Sample> {
    fn stream_spec(&self) -> &StreamSpec;
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError>;
    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError>;

    fn write_exact(&mut self, mut buffer: &[S]) -> Result<(), SyphonError> {
        let block_size = self.stream_spec().block_size;
        if buffer.len() % block_size != 0 {
            return Err(SyphonError::StreamMismatch);
        }

        let mut n_written: usize = 0;
        while !buffer.is_empty() {
            n_written += self.write(buffer)?;
            if n_written == 0 {
                return Err(SyphonError::Empty);
            }

            buffer = &buffer[n_written..];
        }

        Ok(())
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

impl<S: Sample> SampleReader<S> for Box<dyn SampleReader<S>> {
    fn stream_spec(&self) -> &StreamSpec {
        self.as_ref().stream_spec()
    }

    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        self.as_mut().read(buffer)
    }

    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError> {
        self.as_mut().seek(offset)
    }
}

impl<S: Sample> SampleWriter<S> for Box<dyn SampleWriter<S>> {
    fn stream_spec(&self) -> &StreamSpec {
        self.as_ref().stream_spec()
    }

    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        self.as_mut().write(buffer)
    }

    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError> {
        self.as_mut().seek(offset)
    }
}

impl StreamSpec {
    pub fn bytes_per_block(&self) -> usize {
        self.block_size * self.sample_format.byte_size()
    }

    pub fn into_builder(self) -> StreamSpecBuilder {
        StreamSpecBuilder {
            sample_format: Some(self.sample_format),
            n_channels: Some(self.n_channels),
            sample_rate: Some(self.sample_rate),
            block_size: Some(self.block_size),
            n_frames: self.n_frames,
        }
    }
}

impl StreamSpecBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.sample_format.is_none()
            && self.n_channels.is_none()
            && self.sample_rate.is_none()
            && self.block_size.is_none()
            && self.n_frames.is_none()
    }

    pub fn try_build(self) -> Result<StreamSpec, SyphonError> {
        Ok(StreamSpec {
            sample_format: self.sample_format.ok_or(SyphonError::MalformedData)?,
            n_channels: self.n_channels.ok_or(SyphonError::MalformedData)?,
            sample_rate: self.sample_rate.ok_or(SyphonError::MalformedData)?,
            block_size: self.block_size.ok_or(SyphonError::MalformedData)?,
            n_frames: self.n_frames,
        })
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
    pub fn stream_spec(&self) -> &StreamSpec {
        match self {
            Self::I8(reader) => reader.stream_spec(),
            Self::I16(reader) => reader.stream_spec(),
            Self::I32(reader) => reader.stream_spec(),
            Self::I64(reader) => reader.stream_spec(),

            Self::U8(reader) => reader.stream_spec(),
            Self::U16(reader) => reader.stream_spec(),
            Self::U32(reader) => reader.stream_spec(),
            Self::U64(reader) => reader.stream_spec(),

            Self::F32(reader) => reader.stream_spec(),
            Self::F64(reader) => reader.stream_spec(),
        }
    }
}

impl SampleWriterRef {
    pub fn stream_spec(&self) -> &StreamSpec {
        match self {
            Self::I8(writer) => writer.stream_spec(),
            Self::I16(writer) => writer.stream_spec(),
            Self::I32(writer) => writer.stream_spec(),
            Self::I64(writer) => writer.stream_spec(),

            Self::U8(writer) => writer.stream_spec(),
            Self::U16(writer) => writer.stream_spec(),
            Self::U32(writer) => writer.stream_spec(),
            Self::U64(writer) => writer.stream_spec(),

            Self::F32(writer) => writer.stream_spec(),
            Self::F64(writer) => writer.stream_spec(),
        }
    }
}

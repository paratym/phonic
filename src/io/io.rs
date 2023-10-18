use crate::{io::SyphonCodec, Sample, SampleFormat, SyphonError};
use std::io::{Read, Seek, SeekFrom};

pub trait MediaSource: Read + Seek {}

impl<T: Read + Seek> MediaSource for T {}

pub struct FormatDataBuilder {
    pub tracks: Vec<EncodedStreamSpecBuilder>,
}

pub struct FormatReadResult {
    pub track_i: usize,
    pub n_bytes: usize,
}

pub trait FormatReader {
    fn try_format(&mut self) -> Result<(), SyphonError>;
    fn format_data(&self) -> &FormatDataBuilder;
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError>;
    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError>;
}

impl FormatDataBuilder {
    pub fn new() -> Self {
        Self { tracks: Vec::new() }
    }
}

#[derive(Clone, Copy)]
pub struct EncodedStreamSpec {
    pub codec_key: SyphonCodec,
    pub decoded_spec: StreamSpec,
    pub block_size: usize,
    pub byte_len: Option<usize>,
}

#[derive(Clone, Copy, Default)]
pub struct EncodedStreamSpecBuilder {
    pub codec_key: Option<SyphonCodec>,
    pub decoded_spec: StreamSpecBuilder,
    pub block_size: Option<usize>,
    pub byte_len: Option<usize>,
}

pub trait EncodedStreamReader: Read + Seek {
    fn stream_spec(&self) -> &EncodedStreamSpec;
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
            decoded_spec: self.decoded_spec.try_build()?,
            block_size: self.block_size.ok_or(SyphonError::MalformedData)?,
            byte_len: self.byte_len,
        })
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct StreamSpec {
    pub sample_format: SampleFormat,
    pub n_channels: u16,
    pub sample_rate: u32,
    pub block_size: usize,
    pub n_frames: Option<usize>,
}

#[derive(Default, Clone, Copy)]
pub struct StreamSpecBuilder {
    pub sample_format: Option<SampleFormat>,
    pub n_channels: Option<u16>,
    pub sample_rate: Option<u32>,
    pub block_size: Option<usize>,
    pub n_frames: Option<usize>,
}

pub trait SampleReader<S: Sample> {
    fn stream_spec(&self) -> &StreamSpec;
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;

    fn read_exact(&mut self, mut buffer: &mut [S]) -> Result<(), SyphonError> {
        let block_size = self.stream_spec().block_size;
        if buffer.len() % block_size != 0 {
            return Err(SyphonError::StreamMismatch);
        }

        while !buffer.is_empty() {
            let n_read = self.read(buffer)?;
            if n_read == 0 {
                return Err(SyphonError::Empty);
            } else if n_read % block_size != 0 {
                return Err(SyphonError::MalformedData);
            }

            buffer = &mut buffer[n_read..];
        }

        Ok(())
    }
}

pub trait SampleWriter<S: Sample> {
    fn stream_spec(&self) -> &StreamSpec;
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError>;

    fn write_exact(&mut self, mut buffer: &[S]) -> Result<(), SyphonError> {
        let block_size = self.stream_spec().block_size;
        if buffer.len() % block_size != 0 {
            return Err(SyphonError::StreamMismatch);
        }

        while !buffer.is_empty() {
            let n_written = self.write(buffer)?;
            if n_written == 0 {
                return Err(SyphonError::Empty);
            } else if n_written % block_size != 0 {
                return Err(SyphonError::MalformedData);
            }

            buffer = &buffer[n_written..];
        }

        Ok(())
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

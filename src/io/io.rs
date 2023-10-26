use crate::{
    dsp::adapters::SampleTypeAdapter,
    io::{utils::Track, SyphonCodec, SyphonFormat},
    SignalReader, SignalSpec, SignalSpecBuilder, SignalWriter, SyphonError, IntoKnownSample, FromKnownSample,
};
use std::io::{Read, Write};

#[derive(Clone)]
pub struct FormatData {
    pub format: Option<SyphonFormat>,
    pub tracks: Vec<StreamSpecBuilder>,
    // pub metadata: Metadata,
}

impl FormatData {
    pub fn new() -> Self {
        Self {
            format: None,
            tracks: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.format.is_none() && self.tracks.is_empty()
    }

    pub fn format(mut self, format: SyphonFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn track(mut self, track: StreamSpecBuilder) -> Self {
        self.tracks.push(track);
        self
    }

    pub fn fill(mut self) -> Result<Self, SyphonError> {
        if let Some(format) = self.format {
            format.fill_data(&mut self)?;
        }

        Ok(self)
    }

    pub fn write(mut self, writer: Box<dyn Write>) -> Result<Box<dyn FormatWriter>, SyphonError> {
        self = self.fill()?;
        self.format
            .ok_or(SyphonError::Unsupported)?
            .writer(writer, self)
    }
}

pub trait Format {
    fn format_data(&self) -> &FormatData;

    fn default_track_i(&self) -> Option<usize> {
        self.format_data()
            .tracks
            .iter()
            .position(|track| track.codec.is_some())
    }

    fn default_track(&self) -> Option<&StreamSpecBuilder> {
        self.default_track_i()
            .and_then(|i| self.format_data().tracks.get(i))
    }

    fn into_track(self, i: usize) -> Result<Track<Self>, SyphonError>
    where
        Self: Sized,
    {
        Track::from_format(self, i)
    }

    fn into_default_track(self) -> Result<Track<Self>, SyphonError>
    where
        Self: Sized,
    {
        self.default_track_i()
            .ok_or(SyphonError::NotFound)
            .and_then(|i| self.into_track(i))
    }
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
pub struct StreamSpec {
    pub codec: Option<SyphonCodec>,
    pub decoded_spec: SignalSpecBuilder,
    pub block_size: usize,
    pub byte_len: Option<u64>,
}

#[derive(Clone, Copy, Default)]
pub struct StreamSpecBuilder {
    pub codec: Option<SyphonCodec>,
    pub decoded_spec: SignalSpecBuilder,
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

    fn try_from(mut builder: StreamSpecBuilder) -> Result<Self, Self::Error> {
        if let Some(codec) = builder.codec {
            codec.fill_spec(&mut builder)?;
        }

        if builder
            .block_size
            .zip(builder.byte_len)
            .map_or(false, |(b, n)| n % b as u64 != 0)
        {
            return Err(SyphonError::InvalidData);
        }

        Ok(Self {
            codec: builder.codec,
            decoded_spec: builder.decoded_spec,
            block_size: builder.block_size.ok_or(SyphonError::InvalidData)?,
            byte_len: builder.byte_len,
        })
    }
}

impl StreamSpecBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.codec.is_none()
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
        self.codec = Some(codec);
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

    pub fn fill(mut self) -> Result<Self, SyphonError> {
        if let Some(codec) = self.codec {
            codec.fill_spec(&mut self)?;
        }

        Ok(self)
    }

    pub fn build(self) -> Result<StreamSpec, SyphonError> {
        self.try_into()
    }
}

impl From<StreamSpec> for StreamSpecBuilder {
    fn from(spec: StreamSpec) -> Self {
        Self {
            codec: spec.codec,
            decoded_spec: spec.decoded_spec,
            block_size: Some(spec.block_size),
            byte_len: spec.byte_len,
        }
    }
}

pub trait Stream {
    fn spec(&self) -> &StreamSpec;
}

pub trait StreamReader: Stream + Read {
    fn decoder(self) -> Result<SignalReaderRef, SyphonError>
    where
        Self: Sized + 'static,
    {
        self.spec()
            .codec
            .ok_or(SyphonError::Unsupported)?
            .decoder(Box::new(self))
    }
}

pub trait StreamWriter: Stream + Write {
    fn encoder(self) -> Result<SignalWriterRef, SyphonError>
    where
        Self: Sized + 'static,
    {
        self.spec()
            .codec
            .ok_or(SyphonError::Unsupported)?
            .encoder(Box::new(self))
    }
}

impl<T: Stream + Read> StreamReader for T {}

impl Stream for Box<dyn StreamReader> {
    fn spec(&self) -> &StreamSpec {
        self.as_ref().spec()
    }
}

impl<T: Stream + Write> StreamWriter for T {}

impl Stream for Box<dyn StreamWriter> {
    fn spec(&self) -> &StreamSpec {
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
        }
    };
}

impl_signal_ref!(SignalReaderRef);
impl_signal_ref!(SignalWriterRef);

impl SignalReaderRef {
    pub fn adapt_sample_type<O: FromKnownSample + 'static>(self) -> Box<dyn SignalReader<O>> {
        match_signal_ref!(
            self,
            Self,
            signal,
            Box::new(SampleTypeAdapter::from_signal(signal))
        )
    }
}

impl SignalWriterRef {
    pub fn adapt_sample_type<O: IntoKnownSample + 'static>(self) -> Box<dyn SignalWriter<O>> {
        match_signal_ref!(
            self,
            Self,
            signal,
            Box::new(SampleTypeAdapter::from_signal(signal))
        )
    }
}

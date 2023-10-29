use crate::{
    dsp::adapters::SampleTypeAdapter,
    io::{utils::Track, SyphonCodec, SyphonFormat},
    FromKnownSample, IntoKnownSample, Signal, SignalReader, SignalSpec, SignalSpecBuilder,
    SignalWriter, SyphonError,
};
use std::io::{Read, Write};

#[derive(Clone)]
pub struct FormatData {
    pub format: SyphonFormat,
    pub tracks: Box<[StreamSpec]>,
}

pub struct FormatDataBuilder {
    pub format: Option<SyphonFormat>,
    pub tracks: Vec<StreamSpecBuilder>,
}

impl FormatData {
    pub fn builder() -> FormatDataBuilder {
        FormatDataBuilder::new()
    }

    pub fn writer(self, sink: Box<dyn Write>) -> Result<Box<dyn FormatWriter>, SyphonError> {
        let format = self.format;
        format.writer(sink, self)
    }
}

impl TryFrom<FormatDataBuilder> for FormatData {
    type Error = SyphonError;

    fn try_from(mut builder: FormatDataBuilder) -> Result<Self, Self::Error> {
        builder.fill()?;

        Ok(Self {
            format: builder.format.unwrap_or(SyphonFormat::Unknown),
            tracks: builder
                .tracks
                .into_iter()
                .map(|track| track.try_into())
                .collect::<Result<_, _>>()?,
        })
    }
}

impl FormatDataBuilder {
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

    pub fn fill(&mut self) -> Result<(), SyphonError> {
        if let Some(format) = self.format {
            format.fill_data(self)?;
        }

        for track in &mut self.tracks {
            track.fill()?;
        }

        Ok(())
    }

    pub fn build(self) -> Result<FormatData, SyphonError> {
        self.try_into()
    }
}

pub trait Format {
    fn format_data(&self) -> &FormatData;

    fn default_track(&self) -> Result<usize, SyphonError> {
        self.format_data()
            .tracks
            .iter()
            .position(|track| track.codec != SyphonCodec::Unknown)
            .ok_or(SyphonError::NotFound)
    }

    fn into_track_stream(self, i: usize) -> Result<Track<Self>, SyphonError>
    where
        Self: Sized,
    {
        let spec = *self
            .format_data()
            .tracks
            .get(i)
            .ok_or(SyphonError::NotFound)?;

        Ok(Track::new(self, i, spec))
    }

    fn into_default_stream(self) -> Result<Track<Self>, SyphonError>
    where
        Self: Sized,
    {
        self.default_track().and_then(|i| self.into_track_stream(i))
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
    fn finalize(&mut self) -> Result<(), SyphonError> {
        self.flush()
    }
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
    pub codec: SyphonCodec,
    pub decoded_spec: SignalSpec,
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
        builder.fill()?;
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

    fn into_decoder(self) -> Result<SignalReaderRef, SyphonError>
    where
        Self: Sized + 'static,
    {
        let codec = self.spec().codec;
        codec.decoder(Box::new(self))
    }
}

pub trait StreamWriter: Stream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SyphonError>;

    fn into_encoder(self) -> Result<SignalWriterRef, SyphonError>
    where
        Self: Sized + 'static,
    {
        let codec = self.spec().codec;
        codec.encoder(Box::new(self))
    }
}

impl<T: Stream + Read> StreamReader for T {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SyphonError> {
        let block_size = self.spec().block_size;
        match self.read(buf) {
            Ok(n) if n % block_size == 0 => Ok(n / block_size),
            Ok(_) => todo!(),
            Err(e) => Err(e.into()),
        }
    }
}

impl Stream for Box<dyn StreamReader> {
    fn spec(&self) -> &StreamSpec {
        self.as_ref().spec()
    }
}

impl StreamReader for Box<dyn StreamReader> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SyphonError> {
        self.as_mut().read(buf)
    }

    fn into_decoder(self) -> Result<SignalReaderRef, SyphonError>
    where
        Self: Sized + 'static,
    {
        let codec = self.spec().codec;
        codec.decoder(self)
    }
}

impl<T: Stream + Write> StreamWriter for T {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SyphonError> {
        let block_size = self.spec().block_size;
        match self.write(buf) {
            Ok(n) if n % block_size == 0 => Ok(n / block_size),
            Ok(_) => todo!(),
            Err(e) => Err(e.into()),
        }
    }
}

impl Stream for Box<dyn StreamWriter> {
    fn spec(&self) -> &StreamSpec {
        self.as_ref().spec()
    }
}

impl StreamWriter for Box<dyn StreamWriter> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SyphonError> {
        self.as_mut().write(buf)
    }

    fn into_encoder(self) -> Result<SignalWriterRef, SyphonError>
    where
        Self: Sized + 'static,
    {
        let codec = self.spec().codec;
        codec.encoder(self)
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

macro_rules! impl_try_into_inner {
    ($self:ident, $inner:ident, $name:ident, $sample:ty, $variant:ident) => {
        pub fn $name(self) -> Result<Box<dyn $inner<$sample>>, SyphonError> {
            use $self::*;

            match self {
                $variant(signal) => Ok(signal),
                _ => Err(SyphonError::SignalMismatch),
            }
        }
    };
}

macro_rules! impl_signal_ref {
    ($self:ident, $inner:ident) => {
        impl $self {
            pub fn spec(&self) -> &SignalSpec {
                match_signal_ref!(self, Self, signal, signal.spec())
            }

            pub fn adapt_sample_type<O: FromKnownSample + IntoKnownSample + 'static>(
                self,
            ) -> Box<dyn $inner<O>> {
                match_signal_ref!(
                    self,
                    Self,
                    signal,
                    Box::new(SampleTypeAdapter::from_signal(signal))
                )
            }

            impl_try_into_inner!($self, $inner, unwrap_i8_signal, i8, I8);
            impl_try_into_inner!($self, $inner, unwrap_i16_signal, i16, I16);
            impl_try_into_inner!($self, $inner, unwrap_i32_signal, i32, I32);
            impl_try_into_inner!($self, $inner, unwrap_i64_signal, i64, I64);

            impl_try_into_inner!($self, $inner, unwrap_u8_signal, u8, U8);
            impl_try_into_inner!($self, $inner, unwrap_u16_signal, u16, U16);
            impl_try_into_inner!($self, $inner, unwrap_u32_signal, u32, U32);
            impl_try_into_inner!($self, $inner, unwrap_u64_signal, u64, U64);

            impl_try_into_inner!($self, $inner, unwrap_f32_signal, f32, F32);
            impl_try_into_inner!($self, $inner, unwrap_f64_signal, f64, F64);
        }
    };
}

impl_signal_ref!(SignalReaderRef, SignalReader);
impl_signal_ref!(SignalWriterRef, SignalWriter);

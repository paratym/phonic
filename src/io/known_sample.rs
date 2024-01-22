use crate::{
    signal::{
        adapters::SampleTypeAdapter, FromSample, IntoSample, Sample, Signal, SignalReader,
        SignalSpec, SignalWriter,
    },
    SyphonError,
};
use std::{any::TypeId, borrow::BorrowMut, mem::size_of};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KnownSampleType {
    I8,
    I16,
    // I24,
    I32,
    // I48,
    I64,

    U8,
    U16,
    // U24,
    U32,
    // U48,
    U64,

    F32,
    F64,
}

impl KnownSampleType {
    pub fn byte_size(self) -> usize {
        match self {
            Self::I8 => size_of::<i8>(),
            Self::I16 => size_of::<i16>(),
            Self::I32 => size_of::<i32>(),
            Self::I64 => size_of::<i64>(),
            Self::U8 => size_of::<u8>(),
            Self::U16 => size_of::<u16>(),
            Self::U32 => size_of::<u32>(),
            Self::U64 => size_of::<u64>(),
            Self::F32 => size_of::<f32>(),
            Self::F64 => size_of::<f64>(),
        }
    }
}

pub trait FromKnownSample:
    Sample
    + FromSample<i8>
    + FromSample<i16>
    + FromSample<i32>
    + FromSample<i64>
    + FromSample<u8>
    + FromSample<u16>
    + FromSample<u32>
    + FromSample<u64>
    + FromSample<f32>
    + FromSample<f64>
{
}

pub trait IntoKnownSample:
    Sample
    + IntoSample<i8>
    + IntoSample<i16>
    + IntoSample<i32>
    + IntoSample<i64>
    + IntoSample<u8>
    + IntoSample<u16>
    + IntoSample<u32>
    + IntoSample<u64>
    + IntoSample<f32>
    + IntoSample<f64>
{
}

impl TryFrom<TypeId> for KnownSampleType {
    type Error = SyphonError;

    fn try_from(id: TypeId) -> Result<Self, Self::Error> {
        if id == TypeId::of::<u8>() {
            Ok(Self::U8)
        } else if id == TypeId::of::<u16>() {
            Ok(Self::U16)
        } else if id == TypeId::of::<u32>() {
            Ok(Self::U32)
        } else if id == TypeId::of::<u64>() {
            Ok(Self::U64)
        } else if id == TypeId::of::<i8>() {
            Ok(Self::I8)
        } else if id == TypeId::of::<i16>() {
            Ok(Self::I16)
        } else if id == TypeId::of::<i32>() {
            Ok(Self::I32)
        } else if id == TypeId::of::<i64>() {
            Ok(Self::I64)
        } else if id == TypeId::of::<f32>() {
            Ok(Self::F32)
        } else if id == TypeId::of::<f64>() {
            Ok(Self::F64)
        } else {
            Err(SyphonError::Unsupported)
        }
    }
}

impl From<KnownSampleType> for TypeId {
    fn from(value: KnownSampleType) -> Self {
        match value {
            KnownSampleType::I8 => TypeId::of::<i8>(),
            KnownSampleType::I16 => TypeId::of::<i16>(),
            KnownSampleType::I32 => TypeId::of::<i32>(),
            KnownSampleType::I64 => TypeId::of::<i64>(),
            KnownSampleType::U8 => TypeId::of::<u8>(),
            KnownSampleType::U16 => TypeId::of::<u16>(),
            KnownSampleType::U32 => TypeId::of::<u32>(),
            KnownSampleType::U64 => TypeId::of::<u64>(),
            KnownSampleType::F32 => TypeId::of::<f32>(),
            KnownSampleType::F64 => TypeId::of::<f64>(),
        }
    }
}

impl<S> FromKnownSample for S where
    S: Sample
        + FromSample<i8>
        + FromSample<i16>
        + FromSample<i32>
        + FromSample<i64>
        + FromSample<u8>
        + FromSample<u16>
        + FromSample<u32>
        + FromSample<u64>
        + FromSample<f32>
        + FromSample<f64>
{
}

impl<S> IntoKnownSample for S where
    S: Sample
        + IntoSample<i8>
        + IntoSample<i16>
        + IntoSample<i32>
        + IntoSample<i64>
        + IntoSample<u8>
        + IntoSample<u16>
        + IntoSample<u32>
        + IntoSample<u64>
        + IntoSample<f32>
        + IntoSample<f64>
{
}

pub enum TaggedSignalReader {
    I8(Box<dyn SignalReader<Sample = i8>>),
    I16(Box<dyn SignalReader<Sample = i16>>),
    I32(Box<dyn SignalReader<Sample = i32>>),
    I64(Box<dyn SignalReader<Sample = i64>>),

    U8(Box<dyn SignalReader<Sample = u8>>),
    U16(Box<dyn SignalReader<Sample = u16>>),
    U32(Box<dyn SignalReader<Sample = u32>>),
    U64(Box<dyn SignalReader<Sample = u64>>),

    F32(Box<dyn SignalReader<Sample = f32>>),
    F64(Box<dyn SignalReader<Sample = f64>>),
}

pub enum TaggedSignalWriter {
    I8(Box<dyn SignalWriter<Sample = i8>>),
    I16(Box<dyn SignalWriter<Sample = i16>>),
    I32(Box<dyn SignalWriter<Sample = i32>>),
    I64(Box<dyn SignalWriter<Sample = i64>>),

    U8(Box<dyn SignalWriter<Sample = u8>>),
    U16(Box<dyn SignalWriter<Sample = u16>>),
    U32(Box<dyn SignalWriter<Sample = u32>>),
    U64(Box<dyn SignalWriter<Sample = u64>>),

    F32(Box<dyn SignalWriter<Sample = f32>>),
    F64(Box<dyn SignalWriter<Sample = f64>>),
}

macro_rules! match_signal_ref {
    ($ref:ident, $self:ident, $inner:pat, $rhs:expr) => {
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

macro_rules! impl_unwrap {
    ($self:ident, $inner:ident, $name:ident, $sample:ty, $variant:ident) => {
        pub fn $name(self) -> Result<Box<dyn $inner<Sample = $sample>>, SyphonError> {
            match self {
                $self::$variant(signal) => Ok(signal),
                _ => Err(SyphonError::SignalMismatch),
            }
        }
    };
}

macro_rules! impl_from_inner {
    ($self: ident, $inner:ident, $sample:ty, $variant:ident) => {
        impl From<Box<dyn $inner<Sample = $sample>>> for $self {
            fn from(signal: Box<dyn $inner<Sample = $sample>>) -> Self {
                Self::$variant(signal)
            }
        }
    };
}

macro_rules! impl_signal_ref {
    ($self:ident, $tag_inner:ident) => {
        impl $self {
            pub fn spec(&self) -> SignalSpec {
                match_signal_ref!(self, Self, ref signal, (*signal.spec()).into())
            }

            impl_unwrap!($self, $tag_inner, unwrap_i8_signal, i8, I8);
            impl_unwrap!($self, $tag_inner, unwrap_i16_signal, i16, I16);
            impl_unwrap!($self, $tag_inner, unwrap_i32_signal, i32, I32);
            impl_unwrap!($self, $tag_inner, unwrap_i64_signal, i64, I64);

            impl_unwrap!($self, $tag_inner, unwrap_u8_signal, u8, U8);
            impl_unwrap!($self, $tag_inner, unwrap_u16_signal, u16, U16);
            impl_unwrap!($self, $tag_inner, unwrap_u32_signal, u32, U32);
            impl_unwrap!($self, $tag_inner, unwrap_u64_signal, u64, U64);

            impl_unwrap!($self, $tag_inner, unwrap_f32_signal, f32, F32);
            impl_unwrap!($self, $tag_inner, unwrap_f64_signal, f64, F64);
        }

        impl_from_inner!($self, $tag_inner, i8, I8);
        impl_from_inner!($self, $tag_inner, i16, I16);
        impl_from_inner!($self, $tag_inner, i32, I32);
        impl_from_inner!($self, $tag_inner, i64, I64);

        impl_from_inner!($self, $tag_inner, u8, U8);
        impl_from_inner!($self, $tag_inner, u16, U16);
        impl_from_inner!($self, $tag_inner, u32, U32);
        impl_from_inner!($self, $tag_inner, u64, U64);

        impl_from_inner!($self, $tag_inner, f32, F32);
        impl_from_inner!($self, $tag_inner, f64, F64);
    };
}

impl_signal_ref!(TaggedSignalReader, SignalReader);
impl TaggedSignalReader {
    pub fn adapt_sample_type<S: FromKnownSample + 'static>(
        self,
    ) -> Box<dyn SignalReader<Sample = S>> {
        match_signal_ref!(self, Self, signal, Box::new(SampleTypeAdapter::new(signal)))
    }
}

impl_signal_ref!(TaggedSignalWriter, SignalWriter);
impl TaggedSignalWriter {
    pub fn adapt_sample_type<S: IntoKnownSample + 'static>(
        self,
    ) -> Box<dyn SignalWriter<Sample = S>> {
        match_signal_ref!(self, Self, signal, Box::new(SampleTypeAdapter::new(signal)))
    }
}

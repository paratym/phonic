use crate::{FromSample, IntoSample, Sample};
use std::{any::TypeId, mem::size_of};
use phonic_core::PhonicError;

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

pub trait KnownSample: Sample {
    const TYPE: KnownSampleType;
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

macro_rules! impl_known_sample {
    ($self:ty, $type:ident) => {
        impl KnownSample for $self {
            const TYPE: KnownSampleType = KnownSampleType::$type;
        }
    };
}

impl_known_sample!(i8, I8);
impl_known_sample!(i16, I16);
impl_known_sample!(i32, I32);
impl_known_sample!(i64, I64);
impl_known_sample!(u8, U8);
impl_known_sample!(u16, U16);
impl_known_sample!(u32, U32);
impl_known_sample!(u64, U64);
impl_known_sample!(f32, F32);
impl_known_sample!(f64, F64);

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

impl TryFrom<TypeId> for KnownSampleType {
    type Error = PhonicError;

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
            Err(PhonicError::Unsupported)
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

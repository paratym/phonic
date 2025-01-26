use crate::dynamic::DynSignal;
use phonic_signal::{PhonicError, Sample, Signal, SignalSpec};
use std::{any::TypeId, mem::size_of};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KnownSampleType {
    I8,
    I16,
    I32,
    I64,

    U8,
    U16,
    U32,
    U64,

    F32,
    F64,
}

pub trait KnownSample: Sample {
    const TYPE: KnownSampleType;
}

pub enum TaggedSignal {
    I8(Box<dyn DynSignal<Sample = i8>>),
    I16(Box<dyn DynSignal<Sample = i16>>),
    I32(Box<dyn DynSignal<Sample = i32>>),
    I64(Box<dyn DynSignal<Sample = i64>>),

    U8(Box<dyn DynSignal<Sample = u8>>),
    U16(Box<dyn DynSignal<Sample = u16>>),
    U32(Box<dyn DynSignal<Sample = u32>>),
    U64(Box<dyn DynSignal<Sample = u64>>),

    F32(Box<dyn DynSignal<Sample = f32>>),
    F64(Box<dyn DynSignal<Sample = f64>>),
}

impl KnownSampleType {
    pub fn id(self) -> TypeId {
        match self {
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

    pub fn size(self) -> usize {
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

    pub fn align(self) -> usize {
        match self {
            Self::I8 => align_of::<i8>(),
            Self::I16 => align_of::<i16>(),
            Self::I32 => align_of::<i32>(),
            Self::I64 => align_of::<i64>(),
            Self::U8 => align_of::<u8>(),
            Self::U16 => align_of::<u16>(),
            Self::U32 => align_of::<u32>(),
            Self::U64 => align_of::<u64>(),
            Self::F32 => align_of::<f32>(),
            Self::F64 => align_of::<f64>(),
        }
    }
}

impl TryFrom<TypeId> for KnownSampleType {
    type Error = PhonicError;

    fn try_from(id: TypeId) -> Result<Self, Self::Error> {
        Ok(match id {
            id if id == TypeId::of::<i8>() => Self::I8,
            id if id == TypeId::of::<i16>() => Self::I16,
            id if id == TypeId::of::<i32>() => Self::I32,
            id if id == TypeId::of::<i64>() => Self::I64,
            id if id == TypeId::of::<u8>() => Self::U8,
            id if id == TypeId::of::<u16>() => Self::U16,
            id if id == TypeId::of::<u32>() => Self::U32,
            id if id == TypeId::of::<u64>() => Self::U64,
            id if id == TypeId::of::<f32>() => Self::F32,
            id if id == TypeId::of::<f64>() => Self::F64,
            _ => return Err(PhonicError::Unsupported),
        })
    }
}

impl From<KnownSampleType> for TypeId {
    fn from(value: KnownSampleType) -> Self {
        value.id()
    }
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

#[macro_export]
macro_rules! match_tagged_signal {
    ($signal:ident, $inner:pat => $rhs:expr) => {
        match $signal {
            $crate::dynamic::TaggedSignal::I8($inner) => $rhs,
            $crate::dynamic::TaggedSignal::I16($inner) => $rhs,
            $crate::dynamic::TaggedSignal::I32($inner) => $rhs,
            $crate::dynamic::TaggedSignal::I64($inner) => $rhs,
            $crate::dynamic::TaggedSignal::U8($inner) => $rhs,
            $crate::dynamic::TaggedSignal::U16($inner) => $rhs,
            $crate::dynamic::TaggedSignal::U32($inner) => $rhs,
            $crate::dynamic::TaggedSignal::U64($inner) => $rhs,
            $crate::dynamic::TaggedSignal::F32($inner) => $rhs,
            $crate::dynamic::TaggedSignal::F64($inner) => $rhs,
        }
    };
}

macro_rules! impl_unwrap {
    ($name:ident, $sample:ty, $variant:ident) => {
        pub fn $name(self) -> Option<Box<dyn DynSignal<Sample = $sample>>> {
            match self {
                Self::$variant(signal) => Some(signal),
                _ => None,
            }
        }
    };
}

macro_rules! impl_from_inner {
    ($sample:ty, $variant:ident) => {
        impl From<Box<dyn DynSignal<Sample = $sample>>> for TaggedSignal {
            fn from(signal: Box<dyn DynSignal<Sample = $sample>>) -> Self {
                Self::$variant(signal)
            }
        }
    };
}

impl TaggedSignal {
    impl_unwrap!(unwrap_i8, i8, I8);
    impl_unwrap!(unwrap_i16, i16, I16);
    impl_unwrap!(unwrap_i32, i32, I32);
    impl_unwrap!(unwrap_i64, i64, I64);

    impl_unwrap!(unwrap_u8, u8, U8);
    impl_unwrap!(unwrap_u16, u16, U16);
    impl_unwrap!(unwrap_u32, u32, U32);
    impl_unwrap!(unwrap_u64, u64, U64);

    impl_unwrap!(unwrap_f32, f32, F32);
    impl_unwrap!(unwrap_f64, f64, F64);

    pub fn spec(&self) -> &SignalSpec {
        match_tagged_signal!(self, ref signal => signal.spec())
    }

    pub fn sample_type(&self) -> KnownSampleType {
        match self {
            TaggedSignal::I8(_) => KnownSampleType::I8,
            TaggedSignal::I16(_) => KnownSampleType::I16,
            TaggedSignal::I32(_) => KnownSampleType::I32,
            TaggedSignal::I64(_) => KnownSampleType::I64,
            TaggedSignal::U8(_) => KnownSampleType::U8,
            TaggedSignal::U16(_) => KnownSampleType::U16,
            TaggedSignal::U32(_) => KnownSampleType::U32,
            TaggedSignal::U64(_) => KnownSampleType::U64,
            TaggedSignal::F32(_) => KnownSampleType::F32,
            TaggedSignal::F64(_) => KnownSampleType::F64,
        }
    }
}

impl_from_inner!(i8, I8);
impl_from_inner!(i16, I16);
impl_from_inner!(i32, I32);
impl_from_inner!(i64, I64);

impl_from_inner!(u8, U8);
impl_from_inner!(u16, U16);
impl_from_inner!(u32, U32);
impl_from_inner!(u64, U64);

impl_from_inner!(f32, F32);
impl_from_inner!(f64, F64);

use crate::{
    dsp::adapters::SampleTypeAdapter, Signal, SignalReader, SignalSpec,
    SignalWriter, SyphonError, KnownSample
};

// macro_rules! impl_upcast {
//     ($name:ident, $inner:ident, $sample:ty) => {
//         fn $name(self) -> Box<dyn $inner<Sample = $sample>>
//         where
//             Self: Sized + 'static,
//         {
//             Box::new(self)
//         }
//     };
// }

// pub trait DynSignalReader:
//     SignalReader<Sample = i8>
//     + SignalReader<Sample = i16>
//     + SignalReader<Sample = i32>
//     + SignalReader<Sample = i64>
//     + SignalReader<Sample = u8>
//     + SignalReader<Sample = u16>
//     + SignalReader<Sample = u32>
//     + SignalReader<Sample = u64>
//     + SignalReader<Sample = f32>
//     + SignalReader<Sample = f64>
// {
//     impl_upcast!(into_i8_signal, SignalReader, i8);
//     impl_upcast!(into_i16_signal, SignalReader, i16);
//     impl_upcast!(into_i32_signal, SignalReader, i32);
//     impl_upcast!(into_i64_signal, SignalReader, i64);

//     impl_upcast!(into_u8_signal, SignalReader, u8);
//     impl_upcast!(into_u16_signal, SignalReader, u16);
//     impl_upcast!(into_u32_signal, SignalReader, u32);
//     impl_upcast!(into_u64_signal, SignalReader, u64);

//     impl_upcast!(into_f32_signal, SignalReader, f32);
//     impl_upcast!(into_f64_signal, SignalReader, f64);
// }

// pub trait DynSignalWriter:
//     SignalWriter<Sample = i8>
//     + SignalWriter<Sample = i16>
//     + SignalWriter<Sample = i32>
//     + SignalWriter<Sample = i64>
//     + SignalWriter<Sample = u8>
//     + SignalWriter<Sample = u16>
//     + SignalWriter<Sample = u32>
//     + SignalWriter<Sample = u64>
//     + SignalWriter<Sample = f32>
//     + SignalWriter<Sample = f64>
// {
//     impl_upcast!(into_i8_signal, SignalWriter, i8);
//     impl_upcast!(into_i16_signal, SignalWriter, i16);
//     impl_upcast!(into_i32_signal, SignalWriter, i32);
//     impl_upcast!(into_i64_signal, SignalWriter, i64);

//     impl_upcast!(into_u8_signal, SignalWriter, u8);
//     impl_upcast!(into_u16_signal, SignalWriter, u16);
//     impl_upcast!(into_u32_signal, SignalWriter, u32);
//     impl_upcast!(into_u64_signal, SignalWriter, u64);

//     impl_upcast!(into_f32_signal, SignalWriter, f32);
//     impl_upcast!(into_f64_signal, SignalWriter, f64);
// }

// impl<T> DynSignalReader for T where
//     T: SignalReader<Sample = i8>
//         + SignalReader<Sample = i16>
//         + SignalReader<Sample = i32>
//         + SignalReader<Sample = i64>
//         + SignalReader<Sample = u8>
//         + SignalReader<Sample = u16>
//         + SignalReader<Sample = u32>
//         + SignalReader<Sample = u64>
//         + SignalReader<Sample = f32>
//         + SignalReader<Sample = f64>
// {
// }

// impl<T> DynSignalWriter for T where
//     T: SignalWriter<Sample = i8>
//         + SignalWriter<Sample = i16>
//         + SignalWriter<Sample = i32>
//         + SignalWriter<Sample = i64>
//         + SignalWriter<Sample = u8>
//         + SignalWriter<Sample = u16>
//         + SignalWriter<Sample = u32>
//         + SignalWriter<Sample = u64>
//         + SignalWriter<Sample = f32>
//         + SignalWriter<Sample = f64>
// {
// }

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

            pub fn adapt_sample_type<S: KnownSample + 'static>(self) -> Box<dyn $tag_inner<Sample = S>> {
                match_signal_ref!(self, Self, signal, Box::new(SampleTypeAdapter::new(signal)))
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
impl_signal_ref!(TaggedSignalWriter, SignalWriter);

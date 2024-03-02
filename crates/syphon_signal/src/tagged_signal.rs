use crate::{
    adapters::SampleTypeAdapter, FromKnownSample, IntoKnownSample, SignalReader, SignalSpec,
    SignalWriter,
};
use syphon_core::SyphonError;

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

    pub fn pipe(self, writer: &mut TaggedSignalWriter) -> Result<u64, SyphonError> {
        match (self, writer) {
            (Self::I8(mut r), TaggedSignalWriter::I8(w)) => r.pipe(w),
            (Self::I16(mut r), TaggedSignalWriter::I16(w)) => r.pipe(w),
            (Self::I32(mut r), TaggedSignalWriter::I32(w)) => r.pipe(w),
            (Self::I64(mut r), TaggedSignalWriter::I64(w)) => r.pipe(w),
            (Self::U8(mut r), TaggedSignalWriter::U8(w)) => r.pipe(w),
            (Self::U16(mut r), TaggedSignalWriter::U16(w)) => r.pipe(w),
            (Self::U32(mut r), TaggedSignalWriter::U32(w)) => r.pipe(w),
            (Self::U64(mut r), TaggedSignalWriter::U64(w)) => r.pipe(w),
            (Self::F32(mut r), TaggedSignalWriter::F32(w)) => r.pipe(w),
            (Self::F64(mut r), TaggedSignalWriter::F64(w)) => r.pipe(w),

            (r, TaggedSignalWriter::I8(w)) => r.adapt_sample_type().pipe(w),
            (r, TaggedSignalWriter::I16(w)) => r.adapt_sample_type().pipe(w),
            (r, TaggedSignalWriter::I32(w)) => r.adapt_sample_type().pipe(w),
            (r, TaggedSignalWriter::I64(w)) => r.adapt_sample_type().pipe(w),
            (r, TaggedSignalWriter::U8(w)) => r.adapt_sample_type().pipe(w),
            (r, TaggedSignalWriter::U16(w)) => r.adapt_sample_type().pipe(w),
            (r, TaggedSignalWriter::U32(w)) => r.adapt_sample_type().pipe(w),
            (r, TaggedSignalWriter::U64(w)) => r.adapt_sample_type().pipe(w),
            (r, TaggedSignalWriter::F32(w)) => r.adapt_sample_type().pipe(w),
            (r, TaggedSignalWriter::F64(w)) => r.adapt_sample_type().pipe(w),
        }
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

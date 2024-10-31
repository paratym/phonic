use crate::{DynSignal, FromKnownSample, KnownSampleType};
use phonic_core::PhonicError;
use phonic_signal::{adapters::SampleTypeAdapter, Signal, SignalReader, SignalSpec, SignalWriter};

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

#[macro_export]
macro_rules! match_tagged_signal {
    ($signal:ident, $inner:pat => $rhs:expr) => {
        match $signal {
            TaggedSignal::I8($inner) => $rhs,
            TaggedSignal::I16($inner) => $rhs,
            TaggedSignal::I32($inner) => $rhs,
            TaggedSignal::I64($inner) => $rhs,
            TaggedSignal::U8($inner) => $rhs,
            TaggedSignal::U16($inner) => $rhs,
            TaggedSignal::U32($inner) => $rhs,
            TaggedSignal::U64($inner) => $rhs,
            TaggedSignal::F32($inner) => $rhs,
            TaggedSignal::F64($inner) => $rhs,
        }
    };
}

macro_rules! impl_unwrap {
    ($name:ident, $sample:ty, $variant:ident) => {
        pub fn $name(self) -> Result<Box<dyn DynSignal<Sample = $sample>>, PhonicError> {
            match self {
                Self::$variant(signal) => Ok(signal),
                _ => Err(PhonicError::SignalMismatch),
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
    impl_unwrap!(unwrap_i8_signal, i8, I8);
    impl_unwrap!(unwrap_i16_signal, i16, I16);
    impl_unwrap!(unwrap_i32_signal, i32, I32);
    impl_unwrap!(unwrap_i64_signal, i64, I64);

    impl_unwrap!(unwrap_u8_signal, u8, U8);
    impl_unwrap!(unwrap_u16_signal, u16, U16);
    impl_unwrap!(unwrap_u32_signal, u32, U32);
    impl_unwrap!(unwrap_u64_signal, u64, U64);

    impl_unwrap!(unwrap_f32_signal, f32, F32);
    impl_unwrap!(unwrap_f64_signal, f64, F64);

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

    // todo: split into as sample and into sample
    pub fn adapt_sample_type<S: FromKnownSample + 'static>(
        self,
    ) -> Box<dyn SignalReader<Sample = S>> {
        match_tagged_signal!(self, signal => Box::new(<SampleTypeAdapter<_, _>>::new(signal)))
    }

    pub fn copy_n(
        &mut self,
        reader: Self,
        n: u64,
        block: bool,
        adapt: bool,
    ) -> Result<(), PhonicError> {
        match (self, reader) {
            (Self::I8(w), Self::I8(mut r)) => w.copy_n(&mut r, n, block),
            (Self::I16(w), Self::I16(mut r)) => w.copy_n(&mut r, n, block),
            (Self::I32(w), Self::I32(mut r)) => w.copy_n(&mut r, n, block),
            (Self::I64(w), Self::I64(mut r)) => w.copy_n(&mut r, n, block),
            (Self::U8(w), Self::U8(mut r)) => w.copy_n(&mut r, n, block),
            (Self::U16(w), Self::U16(mut r)) => w.copy_n(&mut r, n, block),
            (Self::U32(w), Self::U32(mut r)) => w.copy_n(&mut r, n, block),
            (Self::U64(w), Self::U64(mut r)) => w.copy_n(&mut r, n, block),
            (Self::F32(w), Self::F32(mut r)) => w.copy_n(&mut r, n, block),
            (Self::F64(w), Self::F64(mut r)) => w.copy_n(&mut r, n, block),
            _ if !adapt => Err(PhonicError::SignalMismatch),
            (Self::I8(w), r) => w.copy_n(&mut r.adapt_sample_type(), n, block),
            (Self::I16(w), r) => w.copy_n(&mut r.adapt_sample_type(), n, block),
            (Self::I32(w), r) => w.copy_n(&mut r.adapt_sample_type(), n, block),
            (Self::I64(w), r) => w.copy_n(&mut r.adapt_sample_type(), n, block),
            (Self::U8(w), r) => w.copy_n(&mut r.adapt_sample_type(), n, block),
            (Self::U16(w), r) => w.copy_n(&mut r.adapt_sample_type(), n, block),
            (Self::U32(w), r) => w.copy_n(&mut r.adapt_sample_type(), n, block),
            (Self::U64(w), r) => w.copy_n(&mut r.adapt_sample_type(), n, block),
            (Self::F32(w), r) => w.copy_n(&mut r.adapt_sample_type(), n, block),
            (Self::F64(w), r) => w.copy_n(&mut r.adapt_sample_type(), n, block),
        }
    }

    pub fn copy_all(&mut self, reader: Self, block: bool, adapt: bool) -> Result<(), PhonicError> {
        match self.copy_n(reader, u64::MAX, block, adapt) {
            Err(PhonicError::OutOfBounds) => Ok(()),
            result => result,
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

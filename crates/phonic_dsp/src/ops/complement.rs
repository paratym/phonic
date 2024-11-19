use std::ops::Neg;

use phonic_core::PhonicError;
use phonic_signal::{
    FiniteSignal, IndexedSignal, Sample, Signal, SignalReader, SignalSeeker, SignalSpec,
};

pub trait ComplementSample: Sample {
    fn complement(self) -> Self;
}

pub struct Complement<T> {
    inner: T,
}

impl<T> Complement<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Signal> Signal for Complement<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        self.inner.spec()
    }
}

impl<T: IndexedSignal> IndexedSignal for Complement<T> {
    fn pos(&self) -> u64 {
        self.inner.pos()
    }
}

impl<T: FiniteSignal> FiniteSignal for Complement<T> {
    fn len(&self) -> u64 {
        self.inner.len()
    }
}

impl<T> SignalReader for Complement<T>
where
    T: SignalReader,
    T::Sample: ComplementSample,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let n = self.inner.read(buf)?;
        buf[..n].iter_mut().for_each(|s| *s = s.complement());
        Ok(n)
    }
}

impl<T: SignalSeeker> SignalSeeker for Complement<T> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        self.inner.seek(offset)
    }
}

macro_rules! impl_complement {
    ($sample:ty, $name:ident, $func:expr) => {
        impl ComplementSample for $sample {
            #[inline]
            fn complement(self) -> Self {
                let $name = self;
                $func
            }
        }
    };
}

impl_complement!(i8, s, s.saturating_neg());
impl_complement!(i16, s, s.saturating_neg());
impl_complement!(i32, s, s.saturating_neg());
impl_complement!(i64, s, s.saturating_neg());

impl_complement!(u8, s, todo!());
impl_complement!(u16, s, todo!());
impl_complement!(u32, s, todo!());
impl_complement!(u64, s, todo!());

impl_complement!(f32, s, s.neg());
impl_complement!(f64, s, s.neg());

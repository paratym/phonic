use phonic_core::PhonicError;
use phonic_signal::{
    FiniteSignal, IndexedSignal, Sample, Signal, SignalReader, SignalSeeker, SignalSpec,
};
use std::ops::Mul;

pub struct Gain<T> {
    inner: T,
    amp: f32,
}

pub trait GainSample: Sample {
    fn gain(self, amp: &f32) -> Self;
}

impl<T> Gain<T> {
    pub fn new(inner: T, amp: f32) -> Self {
        Self { inner, amp }
    }

    pub fn new_db(inner: T, db: f32) -> Self {
        let amp = f32::powf(10.0, db / 20.0);
        Self::new(inner, amp)
    }

    pub fn attenuate(inner: T, amp: f32) -> Self {
        Self::new(inner, 1.0 / amp)
    }

    pub fn attenuate_db(inner: T, db: f32) -> Self {
        Self::new_db(inner, -db)
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Signal> Signal for Gain<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        self.inner.spec()
    }
}

impl<T: IndexedSignal> IndexedSignal for Gain<T> {
    fn pos(&self) -> u64 {
        self.inner.pos()
    }
}

impl<T: FiniteSignal> FiniteSignal for Gain<T> {
    fn len(&self) -> u64 {
        self.inner.len()
    }
}

impl<T: SignalReader> SignalReader for Gain<T>
where
    Self::Sample: GainSample,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let n = self.inner.read(buf)?;
        buf[..n].iter_mut().for_each(|s| *s = s.gain(&self.amp));

        Ok(n)
    }
}

impl<T: SignalSeeker> SignalSeeker for Gain<T> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        self.inner.seek(offset)
    }
}

macro_rules! impl_gain_sample {
    ($sample:ty, $name:ident, $amp:ident, $func:expr) => {
        impl GainSample for $sample {
            #[inline]
            fn gain(self, $amp: &f32) -> Self {
                let $name = self;
                $func
            }
        }
    };
}

// TODO: finish sample gain
impl_gain_sample!(f32, s, a, s.mul(a));
impl_gain_sample!(f64, s, a, s.mul(*a as f64));

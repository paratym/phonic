use phonic_signal::{
    FiniteSignal, IndexedSignal, PhonicResult, Sample, Signal, SignalReader, SignalSeeker,
    SignalSpec,
};
use std::ops::Mul;

pub struct Gain<T> {
    inner: T,
    ratio: f32,
}

pub trait GainSample: Sample {
    fn gain(self, amp: &f32) -> Self;
}

impl<T> Gain<T> {
    pub fn new(inner: T, ratio: f32) -> Self {
        Self { inner, ratio }
    }

    pub fn new_db(inner: T, db: f32) -> Self {
        let amp = f32::powf(10.0, db / 20.0);
        Self::new(inner, amp)
    }

    pub fn attenuate(inner: T, ratio: f32) -> Self {
        Self::new(inner, ratio.recip())
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
    fn read(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize> {
        let n = self.inner.read(buf)?;
        buf[..n].iter_mut().for_each(|s| *s = s.gain(&self.ratio));

        Ok(n)
    }
}

impl<T: SignalSeeker> SignalSeeker for Gain<T> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        self.inner.seek(offset)
    }
}

macro_rules! impl_gain {
    ($sample:ident, $self:ident, $ratio:ident, $func:expr) => {
        impl GainSample for $sample {
            #[inline]
            fn gain(self, $ratio: &f32) -> Self {
                let $self = self;
                $func
            }
        }
    };
}

macro_rules! impl_signed_gain {
    ($sample:ident, $float:ident, $self:ident, $ratio:ident) => {
        impl_gain!($sample, $self, $ratio, {
            let whole = $self.saturating_mul($ratio.trunc() as $sample);
            let fract = $self as $float * $ratio.fract() as $float;

            whole.saturating_add(fract as $sample)
        });
    };
}

macro_rules! impl_unsigned_gain {
    ($sample:ident, $float:ident, $self:ident, $ratio:ident) => {
        impl_gain!($sample, $self, $ratio, {
            let abs_ratio = $ratio.abs();
            let base_amp = $self.abs_diff($sample::ORIGIN);

            let whole = base_amp.saturating_mul(abs_ratio.trunc() as $sample);
            let fract = (base_amp as $float * abs_ratio.fract() as $float) as $sample;
            let amp = whole.saturating_add(fract);

            if (*$ratio >= 0.0) == ($self >= Self::ORIGIN) {
                Self::ORIGIN.saturating_add(amp)
            } else {
                Self::ORIGIN.saturating_sub(amp)
            }
        });
    };
}

impl_signed_gain!(i8, f32, s, a);
impl_signed_gain!(i16, f32, s, a);
impl_signed_gain!(i32, f64, s, a);
impl_signed_gain!(i64, f64, s, a);

impl_unsigned_gain!(u8, f32, s, a);
impl_unsigned_gain!(u16, f32, s, a);
impl_unsigned_gain!(u32, f64, s, a);
impl_unsigned_gain!(u64, f64, s, a);

impl_gain!(f32, s, a, s.mul(a));
impl_gain!(f64, s, a, s.mul(*a as f64));

use crate::ops::ComplementSample;
use phonic_core::PhonicError;
use phonic_signal::{FiniteSignal, IndexedSignal, Sample, Signal, SignalReader, SignalSeeker};

pub struct Limit<T: Signal> {
    inner: T,
    range: (T::Sample, T::Sample),
}

pub trait LimitSample: Sample {
    fn limit(self, min: &Self, max: &Self) -> Self;
}

pub trait ClipSample: Sample {
    const RANGE: (Self, Self);

    fn clip(self) -> Self;
}

impl<T: Signal> Limit<T> {
    pub fn new(inner: T, limit: T::Sample) -> Self
    where
        T::Sample: ComplementSample + PartialOrd,
    {
        let complement = limit.complement();
        let range = match limit < complement {
            true => (limit, complement),
            false => (complement, limit),
        };

        Self { inner, range }
    }

    pub fn range(inner: T, min: T::Sample, max: T::Sample) -> Self {
        Self {
            inner,
            range: (min, max),
        }
    }

    pub fn clip(inner: T) -> Self
    where
        T::Sample: ClipSample,
    {
        Self {
            inner,
            range: T::Sample::RANGE,
        }
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Signal> Signal for Limit<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &phonic_signal::SignalSpec {
        self.inner.spec()
    }
}

impl<T: IndexedSignal> IndexedSignal for Limit<T> {
    fn pos(&self) -> u64 {
        self.inner.pos()
    }
}

impl<T: FiniteSignal> FiniteSignal for Limit<T> {
    fn len(&self) -> u64 {
        self.inner.len()
    }
}

impl<T: SignalReader> SignalReader for Limit<T>
where
    Self::Sample: LimitSample,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let n = self.inner.read(buf)?;

        buf[..n]
            .iter_mut()
            .for_each(|s| *s = s.limit(&self.range.0, &self.range.1));

        Ok(n)
    }
}

impl<T: SignalSeeker> SignalSeeker for Limit<T> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        self.inner.seek(offset)
    }
}

impl<T: Sample + PartialOrd> LimitSample for T {
    #[inline]
    fn limit(self, min: &Self, max: &Self) -> Self {
        if self.lt(min) {
            *min
        } else if self.gt(max) {
            *max
        } else {
            self
        }
    }
}

macro_rules! impl_clip_int {
    ($sample:ty) => {
        impl ClipSample for $sample {
            const RANGE: (Self, Self) = (Self::MIN, Self::MAX);

            #[inline]
            fn clip(self) -> Self {
                self
            }
        }
    };
}

macro_rules! impl_clip_float {
    ($sample:ty) => {
        impl ClipSample for $sample {
            const RANGE: (Self, Self) = (-1.0, 1.0);

            #[inline]
            fn clip(self) -> Self {
                self.limit(&Self::RANGE.0, &Self::RANGE.1)
            }
        }
    };
}

impl_clip_int!(i8);
impl_clip_int!(i16);
impl_clip_int!(i32);
impl_clip_int!(i64);

impl_clip_int!(u8);
impl_clip_int!(u16);
impl_clip_int!(u32);
impl_clip_int!(u64);

impl_clip_float!(f32);
impl_clip_float!(f64);
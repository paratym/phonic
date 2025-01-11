use crate::ops::ComplementSample;
use phonic_signal::{delegate_signal, PhonicResult, Sample, Signal, SignalExt, SignalReader};
use std::mem::MaybeUninit;

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
        let range = if limit >= complement {
            (complement, limit)
        } else {
            (limit, complement)
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

delegate_signal! {
    impl<T> * + !Read + !Write for Limit<T> {
        Self as T;

        &self => &self.inner;
        &mut self => &mut self.inner;
    }
}

impl<T: SignalReader> SignalReader for Limit<T>
where
    Self::Sample: LimitSample,
{
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let samples = self.inner.read_init(buf)?;

        samples
            .iter_mut()
            .for_each(|s| *s = s.limit(&self.range.0, &self.range.1));

        Ok(samples.len())
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

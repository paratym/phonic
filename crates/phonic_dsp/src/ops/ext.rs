use crate::ops::{ClipSample, Complement, ComplementSample, Convert, Gain, Limit, Mix};
use phonic_core::PhonicError;
use phonic_signal::{IndexedSignal, Sample, Signal};

pub trait OpSignalExt: Sized + Signal {
    fn complement(self) -> Complement<Self> {
        Complement::new(self)
    }

    fn convert<S: Sample>(self) -> Convert<Self, S> {
        Convert::new(self)
    }

    fn convert_buffered<S: Sample, B>(self, buf: B) -> Convert<Self, S, B> {
        Convert::new_buffered(self, buf)
    }

    fn gain_amp(self, amp: f32) -> Gain<Self> {
        Gain::new(self, amp)
    }

    fn gain_db(self, db: f32) -> Gain<Self> {
        Gain::new_db(self, db)
    }

    fn attenuate(self, amp: f32) -> Gain<Self> {
        Gain::attenuate(self, amp)
    }

    fn attenuate_db(self, db: f32) -> Gain<Self> {
        Gain::attenuate_db(self, db)
    }

    fn limit(self, limit: Self::Sample) -> Limit<Self>
    where
        Self::Sample: ComplementSample + PartialOrd,
    {
        Limit::new(self, limit)
    }

    fn limit_range(self, min: Self::Sample, max: Self::Sample) -> Limit<Self> {
        Limit::range(self, min, max)
    }

    fn clip(self) -> Limit<Self>
    where
        Self::Sample: ClipSample,
    {
        Limit::clip(self)
    }

    fn mix<T>(self, other: T) -> Result<Mix<(Self, T)>, PhonicError>
    where
        Self: IndexedSignal,
        T: IndexedSignal<Sample = Self::Sample>,
    {
        Mix::new((self, other))
    }

    fn mix_buffered<T, B>(self, other: T, buf: B) -> Result<Mix<(Self, T), B>, PhonicError>
    where
        Self: IndexedSignal,
        T: IndexedSignal<Sample = Self::Sample>,
    {
        Mix::new_buffered((self, other), buf)
    }

    fn cancel<T>(self, other: T) -> Result<Mix<(Self, Complement<T>)>, PhonicError>
    where
        Self: IndexedSignal,
        T: IndexedSignal<Sample = Self::Sample>,
        T::Sample: ComplementSample,
    {
        Mix::cancel(self, other)
    }

    fn cancel_buffered<T, B>(
        self,
        other: T,
        buf: B,
    ) -> Result<Mix<(Self, Complement<T>), B>, PhonicError>
    where
        Self: IndexedSignal,
        T: IndexedSignal<Sample = Self::Sample>,
        T::Sample: ComplementSample,
    {
        Mix::cancel_buffered(self, other, buf)
    }
}

impl<T: Signal> OpSignalExt for T {}

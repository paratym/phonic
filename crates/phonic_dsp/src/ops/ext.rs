use crate::ops::{
    ClipSample, Complement, ComplementSample, Convert, DbRatio, Gain, GainSample, Limit, Mix,
};
use num_traits::Inv;
use phonic_signal::{IndexedSignal, PhonicResult, Sample, Signal};

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

    fn gain_amp(
        self,
        ratio: <Self::Sample as GainSample>::Ratio,
    ) -> Gain<Self, <Self::Sample as GainSample>::Ratio>
    where
        Self::Sample: GainSample,
    {
        Gain::new(self, ratio)
    }

    fn gain_db(
        self,
        db: <Self::Sample as GainSample>::Ratio,
    ) -> Gain<Self, <Self::Sample as GainSample>::Ratio>
    where
        Self::Sample: GainSample,
        <Self::Sample as GainSample>::Ratio: DbRatio,
    {
        Gain::new_db(self, db)
    }

    fn attenuate(
        self,
        ratio: <Self::Sample as GainSample>::Ratio,
    ) -> Gain<Self, <Self::Sample as GainSample>::Ratio>
    where
        Self::Sample: GainSample,
        <Self::Sample as GainSample>::Ratio: Inv<Output = <Self::Sample as GainSample>::Ratio>,
    {
        Gain::attenuate(self, ratio)
    }

    fn attenuate_db(
        self,
        db: <Self::Sample as GainSample>::Ratio,
    ) -> Gain<Self, <Self::Sample as GainSample>::Ratio>
    where
        Self::Sample: GainSample,
        <Self::Sample as GainSample>::Ratio:
            DbRatio + Inv<Output = <Self::Sample as GainSample>::Ratio>,
    {
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

    fn mix<T>(self, other: T) -> PhonicResult<Mix<(Self, T)>>
    where
        Self: IndexedSignal,
        T: IndexedSignal<Sample = Self::Sample>,
    {
        Mix::new((self, other))
    }

    fn mix_buffered<T, B>(self, other: T, buf: B) -> PhonicResult<Mix<(Self, T), B>>
    where
        Self: IndexedSignal,
        T: IndexedSignal<Sample = Self::Sample>,
    {
        Mix::new_buffered((self, other), buf)
    }

    fn cancel<T>(self, other: T) -> PhonicResult<Mix<(Self, Complement<T>)>>
    where
        Self: IndexedSignal,
        T: IndexedSignal<Sample = Self::Sample>,
        T::Sample: ComplementSample,
    {
        Mix::cancel(self, other)
    }

    fn cancel_buffered<T, B>(self, other: T, buf: B) -> PhonicResult<Mix<(Self, Complement<T>), B>>
    where
        Self: IndexedSignal,
        T: IndexedSignal<Sample = Self::Sample>,
        T::Sample: ComplementSample,
    {
        Mix::cancel_buffered(self, other, buf)
    }
}

impl<T: Signal> OpSignalExt for T {}

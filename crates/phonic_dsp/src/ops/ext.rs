use crate::ops::{
    ClipSample, Complement, ComplementSample, Convert, DbRatio, Gain, GainSample, Limit, Mix,
    Reciprocal,
};
use phonic_signal::{DefaultSizedBuf, IndexedSignal, PhonicResult, Sample, Signal, SizedBuf};

pub trait DspOpsExt: Sized + Signal {
    fn complement(self) -> Complement<Self> {
        Complement::new(self)
    }

    fn convert<S: Sample>(self) -> Convert<Self, S> {
        let buf = DefaultSizedBuf::uninit();
        Convert::new(self, buf)
    }

    fn convert_buf<S: Sample, B>(self, buf: B) -> Convert<Self, S, B> {
        Convert::new(self, buf)
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
        <Self::Sample as GainSample>::Ratio: Reciprocal,
    {
        Gain::attenuate(self, ratio)
    }

    fn attenuate_db(
        self,
        db: <Self::Sample as GainSample>::Ratio,
    ) -> Gain<Self, <Self::Sample as GainSample>::Ratio>
    where
        Self::Sample: GainSample,
        <Self::Sample as GainSample>::Ratio: DbRatio + Reciprocal,
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
        let buf = DefaultSizedBuf::uninit();
        Mix::new((self, other), buf)
    }

    fn mix_buf<T, B>(self, other: T, buf: B) -> PhonicResult<Mix<(Self, T), B>>
    where
        Self: IndexedSignal,
        T: IndexedSignal<Sample = Self::Sample>,
    {
        Mix::new((self, other), buf)
    }

    fn cancel<T>(self, other: T) -> PhonicResult<Mix<(Self, Complement<T>)>>
    where
        Self: IndexedSignal,
        T: IndexedSignal<Sample = Self::Sample>,
        T::Sample: ComplementSample,
    {
        let buf = DefaultSizedBuf::uninit();
        Mix::cancel(self, other, buf)
    }

    fn cancel_buf<T, B>(self, other: T, buf: B) -> PhonicResult<Mix<(Self, Complement<T>), B>>
    where
        Self: IndexedSignal,
        T: IndexedSignal<Sample = Self::Sample>,
        T::Sample: ComplementSample,
    {
        Mix::cancel(self, other, buf)
    }
}

impl<T: Signal> DspOpsExt for T {}

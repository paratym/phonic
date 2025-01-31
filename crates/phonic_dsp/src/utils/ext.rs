use crate::utils::{Concat, Delay, Repeat, Slice};
use phonic_signal::{
    utils::{IntoDuration, NFrames},
    FiniteSignal, IndexedSignal, PhonicResult, Signal,
};

pub trait DspUtilsExt: Sized + Signal {
    fn concat<T>(self, other: T) -> PhonicResult<Concat<(Self, T)>>
    where
        T: Signal<Sample = Self::Sample>,
    {
        Concat::new((self, other))
    }

    fn delay<D: IntoDuration<NFrames>>(self, duration: D) -> Delay<Self>
    where
        Self: IndexedSignal,
    {
        Delay::new(self, duration)
    }

    fn delay_seeked<D: IntoDuration<NFrames>>(self, duration: D) -> Delay<Self> {
        Delay::new_seeked(self, duration)
    }

    fn repeat_n(self, reps: u32) -> Repeat<Self> {
        Repeat::new(self, reps)
    }

    fn repeat_loop(self) -> Repeat<Self> {
        Repeat::new(self, u32::MAX)
    }

    fn slice(
        self,
        start: impl IntoDuration<NFrames>,
        end: impl IntoDuration<NFrames>,
    ) -> Slice<Self> {
        Slice::range(self, start, end)
    }

    fn slice_from_start(self, end: impl IntoDuration<NFrames>) -> Slice<Self> {
        Slice::from_start(self, end)
    }

    fn slice_from_current(self, end: impl IntoDuration<NFrames>) -> Slice<Self>
    where
        Self: IndexedSignal,
    {
        Slice::from_current(self, end)
    }

    fn slice_from_current_offset(self, offset: impl IntoDuration<NFrames>) -> Slice<Self>
    where
        Self: IndexedSignal,
    {
        Slice::from_current_offset(self, offset)
    }

    fn slice_to_end(self, start: impl IntoDuration<NFrames>) -> Slice<Self>
    where
        Self: FiniteSignal,
    {
        Slice::to_end(self, start)
    }

    fn slice_to_end_offset(self, start: impl IntoDuration<NFrames>) -> Slice<Self>
    where
        Self: FiniteSignal,
    {
        Slice::to_end_offset(self, start)
    }
}

impl<T: Signal> DspUtilsExt for T {}

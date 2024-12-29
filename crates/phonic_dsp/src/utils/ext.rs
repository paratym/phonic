use crate::utils::{Concat, Delay, Repeat, Slice, Split};
use phonic_buf::{DefaultSizedBuf, SizedBuf};
use phonic_signal::{FiniteSignal, IndexedSignal, PhonicResult, Signal, SignalDuration};

pub trait DspUtilsExt: Sized + Signal {
    fn concat<T>(self, other: T) -> PhonicResult<Concat<(Self, T)>>
    where
        T: Signal<Sample = Self::Sample>,
    {
        Concat::new((self, other))
    }

    fn delay<D: SignalDuration>(self, duration: D) -> Delay<Self>
    where
        Self: IndexedSignal,
    {
        Delay::new(self, duration)
    }

    fn delay_seeked<D: SignalDuration>(self, duration: D) -> Delay<Self> {
        Delay::new_seeked(self, duration)
    }

    fn repeat_n(self, reps: u32) -> Repeat<Self> {
        Repeat::new(self, reps)
    }

    fn repeat_loop(self) -> Repeat<Self> {
        Repeat::new(self, u32::MAX)
    }

    fn slice<D: SignalDuration>(self, start: D, end: D) -> Slice<Self> {
        Slice::range(self, start, end)
    }

    fn slice_from_start<D: SignalDuration>(self, end: D) -> Slice<Self> {
        Slice::from_start(self, end)
    }

    fn slice_from_current<D: SignalDuration>(self, end: D) -> Slice<Self>
    where
        Self: IndexedSignal,
    {
        Slice::from_current(self, end)
    }

    fn slice_from_current_offset<D: SignalDuration>(self, offset: D) -> Slice<Self>
    where
        Self: IndexedSignal,
    {
        Slice::from_current_offset(self, offset)
    }

    fn slice_to_end<D: SignalDuration>(self, start: D) -> Slice<Self>
    where
        Self: FiniteSignal,
    {
        Slice::to_end(self, start)
    }

    fn slice_to_end_offset<D: SignalDuration>(self, start: D) -> Slice<Self>
    where
        Self: FiniteSignal,
    {
        Slice::to_end_offset(self, start)
    }

    fn split(self) -> Split<Self> {
        let buf = DefaultSizedBuf::silence();
        Split::new(self, buf)
    }

    fn split_buf<B: AsRef<[Self::Sample]>>(self, buf: B) -> Split<Self, B> {
        Split::new(self, buf)
    }
}

impl<T: Signal> DspUtilsExt for T {}

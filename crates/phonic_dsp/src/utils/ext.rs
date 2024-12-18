use crate::utils::{Concat, Delay, Repeat, Slice, Split};
use phonic_signal::{FiniteSignal, IndexedSignal, PhonicResult, Signal};
use std::time::Duration;

pub trait UtilSignalExt: Sized + Signal {
    fn concat<T>(self, other: T) -> PhonicResult<Concat<(Self, T)>>
    where
        T: Signal<Sample = Self::Sample>,
    {
        Concat::new((self, other))
    }

    fn delay(self, n_frames: u64) -> Delay<Self>
    where
        Self: IndexedSignal,
    {
        Delay::new(self, n_frames)
    }

    fn delay_interleaved(self, n_samples: u64) -> Delay<Self>
    where
        Self: IndexedSignal,
    {
        Delay::new_interleaved(self, n_samples)
    }

    fn delay_duration(self, duration: Duration) -> Delay<Self>
    where
        Self: IndexedSignal,
    {
        Delay::new_duration(self, duration)
    }

    fn delay_seeked(self, n_frames: u64) -> Delay<Self> {
        Delay::new_seeked(self, n_frames)
    }

    fn delay_interleaved_seeked(self, n_samples: u64) -> Delay<Self> {
        Delay::new_interleaved_seeked(self, n_samples)
    }

    fn delay_duration_seeked(self, duration: Duration) -> Delay<Self> {
        Delay::new_duration_seeked(self, duration)
    }

    fn repeat_n(self, reps: u32) -> Repeat<Self> {
        Repeat::new(self, reps)
    }

    fn repeat_loop(self) -> Repeat<Self> {
        Repeat::new(self, u32::MAX)
    }

    fn slice(self, start: u64, end: u64) -> Slice<Self> {
        Slice::new(self, start, end)
    }

    fn slice_interleaved(self, start: u64, end: u64) -> Slice<Self> {
        Slice::new_interleaved(self, start, end)
    }

    fn slice_duration(self, start: Duration, end: Duration) -> Slice<Self> {
        Slice::new_duration(self, start, end)
    }

    fn slice_from_start(self, end: u64) -> Slice<Self> {
        Slice::new_from_start(self, end)
    }

    fn slice_from_start_interleaved(self, end: u64) -> Slice<Self> {
        Slice::new_from_start_interleaved(self, end)
    }

    fn slice_from_start_duration(self, end: Duration) -> Slice<Self> {
        Slice::new_from_start_duration(self, end)
    }

    fn slice_from_current(self, end: u64) -> Slice<Self>
    where
        Self: IndexedSignal,
    {
        Slice::new_from_current(self, end)
    }

    fn slice_from_current_interleaved(self, end: u64) -> Slice<Self>
    where
        Self: IndexedSignal,
    {
        Slice::new_from_current_interleaved(self, end)
    }

    fn slice_from_current_duration(self, end: Duration) -> Slice<Self>
    where
        Self: IndexedSignal,
    {
        Slice::new_from_current_duration(self, end)
    }

    fn slice_to_end(self, start: u64) -> Slice<Self>
    where
        Self: FiniteSignal,
    {
        Slice::new_to_end(self, start)
    }

    fn slice_to_end_interleaved(self, start: u64) -> Slice<Self>
    where
        Self: FiniteSignal,
    {
        Slice::new_to_end_interleaved(self, start)
    }

    fn slice_to_end_duration(self, start: Duration) -> Slice<Self>
    where
        Self: FiniteSignal,
    {
        Slice::new_to_end_duration(self, start)
    }

    fn split(self) -> Split<Self> {
        Split::new(self)
    }
}

impl<T: Signal> UtilSignalExt for T {}

use crate::gen::NullSignal;
use phonic_signal::{
    FiniteSignal, IndexedSignal, PhonicError, PhonicResult, Signal, SignalReader, SignalSeeker,
    SignalSpec, SignalWriter,
};
use std::time::Duration;

pub struct Slice<T> {
    inner: T,
    start: u64,
    end: u64,
    pos: u64,
}

impl<T> Slice<T> {
    pub fn new(inner: T, start: u64, end: u64) -> Self {
        Self {
            inner,
            start,
            end: end.max(start),
            pos: 0,
        }
    }

    pub fn new_interleaved(inner: T, start: u64, end: u64) -> Self
    where
        T: Signal,
    {
        let n_channels = inner.spec().channels.count() as u64;
        Self::new(inner, start / n_channels, end / n_channels)
    }

    pub fn new_duration(inner: T, start: Duration, end: Duration) -> Self
    where
        T: Signal,
    {
        let sample_interval = inner.spec().sample_rate_duration().as_secs_f64();
        let start_frame = start.as_secs_f64() / sample_interval;
        let end_frame = end.as_secs_f64() / sample_interval;
        Self::new(inner, start_frame as u64, end_frame as u64)
    }

    pub fn new_from_start(inner: T, end: u64) -> Self {
        Self::new(inner, 0, end)
    }

    pub fn new_from_start_interleaved(inner: T, end: u64) -> Self
    where
        T: Signal,
    {
        Self::new_interleaved(inner, 0, end)
    }

    pub fn new_from_start_duration(inner: T, end: Duration) -> Self
    where
        T: Signal,
    {
        Self::new_duration(inner, Duration::ZERO, end)
    }

    pub fn new_from_current(inner: T, end: u64) -> Self
    where
        T: IndexedSignal,
    {
        let start = inner.pos();
        Self::new(inner, start, end)
    }

    pub fn new_from_current_interleaved(inner: T, end: u64) -> Self
    where
        T: IndexedSignal,
    {
        let start = inner.pos_interleaved();
        Self::new_interleaved(inner, start, end)
    }

    pub fn new_from_current_duration(inner: T, end: Duration) -> Self
    where
        T: IndexedSignal,
    {
        let start = inner.pos_duration();
        Self::new_duration(inner, start, end)
    }

    pub fn new_to_end(inner: T, start: u64) -> Self
    where
        T: FiniteSignal,
    {
        let end = inner.len();
        Self::new(inner, start, end)
    }

    pub fn new_to_end_interleaved(inner: T, start: u64) -> Self
    where
        T: FiniteSignal,
    {
        let end = inner.len_interleaved();
        Self::new_interleaved(inner, start, end)
    }

    pub fn new_to_end_duration(inner: T, start: Duration) -> Self
    where
        T: FiniteSignal,
    {
        let end = inner.len_duration();
        Self::new_duration(inner, start, end)
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Signal> Signal for Slice<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        self.inner.spec()
    }
}

impl<T: Signal> IndexedSignal for Slice<T> {
    fn pos(&self) -> u64 {
        self.pos
    }
}

impl<T: Signal> FiniteSignal for Slice<T> {
    fn len(&self) -> u64 {
        self.end - self.start
    }
}

impl<T: IndexedSignal + SignalReader> SignalReader for Slice<T> {
    fn read(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize> {
        let n_before = self.start.saturating_sub(self.inner.pos());
        if n_before > 0 {
            let mut null = NullSignal::new(*self.spec());
            null.copy_n_buffered(&mut self.inner, n_before, buf, false)?;
        }

        let buf_len = buf.len().min(self.rem_interleaved() as usize);
        let n = self.inner.read(&mut buf[..buf_len])?;
        self.pos += n as u64;

        Ok(n)
    }
}

impl<T: IndexedSignal + SignalWriter> SignalWriter for Slice<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let n_before = self.start.saturating_sub(self.inner.pos());
        if n_before > 0 {
            let mut null = NullSignal::new(*self.spec());
            self.inner.copy_n(&mut null, n_before, false)?;
        }

        let buf_len = buf.len().min(self.rem_interleaved() as usize);
        let n = self.inner.write(&buf[..buf_len])?;
        self.pos += n as u64;

        Ok(n)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush()
    }
}

impl<T: SignalSeeker> SignalSeeker for Slice<T> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        let pos = self
            .pos()
            .checked_add_signed(offset)
            .ok_or(PhonicError::OutOfBounds)?;

        self.inner.seek(offset)?;
        self.pos = pos;

        Ok(())
    }
}

use phonic_macro::impl_deref_signal;
use phonic_signal::{
    utils::DefaultBuf, FiniteSignal, IndexedSignal, PhonicError, PhonicResult, Signal,
    SignalReader, SignalSeeker, SignalSpec, SignalWriter,
};
use std::time::Duration;

pub struct Slice<T> {
    inner: T,
    start: u64,
    end: u64,
}

impl<T> Slice<T> {
    pub fn new(inner: T, start: u64, end: u64) -> Self {
        debug_assert!(start < end);
        Self { inner, start, end }
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

impl_deref_signal! {
    impl<T> _ + !IndexedSignal + !FiniteSignal for Slice<T> {
        type Target = T;

        &self -> &self.inner;
    }
}

impl<T: IndexedSignal> IndexedSignal for Slice<T> {
    fn pos(&self) -> u64 {
        self.inner.pos().saturating_sub(self.start).min(self.len())
    }
}

impl<T: Signal> FiniteSignal for Slice<T> {
    fn len(&self) -> u64 {
        self.end - self.start
    }
}

impl<T: IndexedSignal + SignalReader> Slice<T> {
    fn read_padding(&mut self, buf: &mut [<Self as Signal>::Sample]) -> PhonicResult<()> {
        let buf_len = buf.len();
        let n_channels = self.spec().channels.count() as usize;

        loop {
            let pos = self.inner.pos();
            let n_before = self.start.saturating_sub(pos);
            if n_before == 0 {
                break Ok(());
            }

            let len = buf_len.min(n_before as usize * n_channels);
            self.inner.read(&mut buf[..len])?;
        }
    }
}

impl<T: IndexedSignal + SignalReader> SignalReader for Slice<T> {
    fn read(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize> {
        self.read_padding(buf)?;

        let len = buf.len().min(self.rem_interleaved() as usize);
        self.inner.read(&mut buf[..len])
    }
}

impl<T: IndexedSignal + SignalWriter> Slice<T> {
    fn write_padding(&mut self) -> PhonicResult<()> {
        let buf = DefaultBuf::default();

        let buf_len = buf.len();
        let n_channels = self.spec().channels.count() as usize;

        loop {
            let pos = self.inner.pos();
            let n_before = self.start.saturating_sub(pos);
            if n_before == 0 {
                break Ok(());
            }

            let len = buf_len.min(n_before as usize * n_channels);
            self.inner.write(&buf[..len])?;
        }
    }
}

impl<T: IndexedSignal + SignalWriter> SignalWriter for Slice<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        self.write_padding()?;

        let len = buf.len().min(self.rem_interleaved() as usize);
        self.inner.write(&buf[..len])
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush()
    }
}

impl<T: IndexedSignal + SignalSeeker> SignalSeeker for Slice<T> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        if self
            .pos()
            .checked_add_signed(offset)
            .is_none_or(|pos| pos > self.len())
        {
            return Err(PhonicError::OutOfBounds);
        }

        self.inner.seek(offset)
    }
}

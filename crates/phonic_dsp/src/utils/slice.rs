use phonic_signal::{
    delegate_signal,
    utils::{copy_exact, NullSignal},
    DefaultSizedBuf, FiniteSignal, IndexedSignal, IntoDuration, NFrames, NSamples, PhonicError,
    PhonicResult, Signal, SignalDuration, SignalExt, SignalReader, SignalSeeker, SignalWriter,
    SizedBuf,
};
use std::mem::MaybeUninit;

pub struct Slice<T> {
    inner: T,
    start: u64,
    end: u64,
}

impl<T: Signal> Slice<T> {
    pub fn range<D: SignalDuration>(inner: T, start: D, end: D) -> Self {
        let NFrames { n_frames: start } = start.into_duration(inner.spec());
        let NFrames { n_frames: end } = end.into_duration(inner.spec());

        Self { inner, start, end }
    }

    pub fn offset<D: SignalDuration>(inner: T, start: D, offset: D) -> Self {
        Self::range(inner, start, start + offset)
    }

    pub fn from_start<D: SignalDuration>(inner: T, end: D) -> Self {
        let start = NFrames::from(0).into_duration(inner.spec());
        Self::range(inner, start, end)
    }

    pub fn from_current<D: SignalDuration>(inner: T, end: D) -> Self
    where
        T: IndexedSignal,
    {
        let start = inner.pos_duration();
        Self::range(inner, start, end)
    }

    pub fn from_current_offset<D: SignalDuration>(inner: T, offset: D) -> Self
    where
        T: IndexedSignal,
    {
        let start = inner.pos_duration();
        Self::offset(inner, start, offset)
    }

    pub fn to_end<D: SignalDuration>(inner: T, start: D) -> Self
    where
        T: FiniteSignal,
    {
        let end = inner.len_duration();
        Self::range(inner, start, end)
    }

    pub fn to_end_offset<D: SignalDuration>(inner: T, offset: D) -> Self
    where
        T: FiniteSignal,
    {
        let end: D = inner.len_duration();
        Self::range(inner, end - offset, end)
    }
}

delegate_signal! {
    impl<T> Signal for Slice<T> {
        Self as T;

        &self => &self.inner;
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
    fn read_padding(&mut self, buf: &mut [MaybeUninit<T::Sample>]) -> PhonicResult<()> {
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
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        self.read_padding(buf)?;

        let NSamples { n_samples } = self.rem_duration();
        let len = buf.len().min(n_samples as usize);

        self.inner.read(&mut buf[..len])
    }
}

impl<T: IndexedSignal + SignalWriter> Slice<T> {
    fn write_padding(&mut self) -> PhonicResult<()> {
        let buf = DefaultSizedBuf::<T::Sample>::silence();

        let buf_len = buf.len();
        let n_channels = self.spec().channels.count() as usize;

        loop {
            let pos = self.inner.pos();
            let n_before = self.start.saturating_sub(pos);
            if n_before == 0 {
                break;
            }

            let len = buf_len.min(n_before as usize * n_channels);
            self.inner.write(&buf[..len])?;
        }

        Ok(())
    }
}

impl<T: IndexedSignal + SignalWriter> SignalWriter for Slice<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        self.write_padding()?;

        let NSamples { n_samples } = self.rem_duration();
        let len = buf.len().min(n_samples as usize);

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

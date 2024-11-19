use crate::gen::NullSignal;
use phonic_core::PhonicError;
use phonic_signal::{
    FiniteSignal, IndexedSignal, Sample, Signal, SignalReader, SignalSeeker, SignalSpec,
    SignalWriter,
};
use std::time::Duration;

pub struct Delay<T> {
    inner: T,
    delay: u64,
    rem_padding: u64,
}

impl<T: Signal> Delay<T> {
    pub fn new(inner: T, n_frames: u64) -> Self
    where
        T: IndexedSignal,
    {
        let rem_padding = if inner.pos() > 0 { 0 } else { n_frames };

        Self {
            inner,
            delay: n_frames,
            rem_padding,
        }
    }

    pub fn new_interleaved(inner: T, n_samples: u64) -> Self
    where
        T: IndexedSignal,
    {
        let n_channels = inner.spec().channels.count() as u64;
        debug_assert_eq!(n_samples % n_channels, 0);

        let frame_delay = n_samples / n_channels;
        Self::new(inner, frame_delay)
    }

    pub fn new_duration(inner: T, duration: Duration) -> Self
    where
        T: IndexedSignal,
    {
        let frame_duration = inner.spec().sample_rate_duration().as_secs_f64();
        let frame_delay = duration.as_secs_f64() / frame_duration;
        Self::new(inner, frame_delay as u64)
    }

    pub fn new_seeked(inner: T, n_frames: u64) -> Self {
        Self {
            inner,
            delay: n_frames,
            rem_padding: 0,
        }
    }

    pub fn new_interleaved_seeked(inner: T, n_samples: u64) -> Self {
        let n_channels = inner.spec().channels.count() as u64;
        debug_assert_eq!(n_samples % n_channels, 0);

        let frame_delay = n_samples / n_channels;
        Self::new_seeked(inner, frame_delay)
    }

    pub fn new_duration_seeked(inner: T, duration: Duration) -> Self {
        let frame_duration = inner.spec().sample_rate_duration().as_secs_f64();
        let frame_delay = duration.as_secs_f64() / frame_duration;
        Self::new_seeked(inner, frame_delay as u64)
    }
}

impl<T> Delay<T> {
    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Signal> Signal for Delay<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        self.inner.spec()
    }
}

impl<T: IndexedSignal> IndexedSignal for Delay<T> {
    fn pos(&self) -> u64 {
        if self.rem_padding > 0 {
            self.delay - self.rem_padding
        } else {
            self.inner.pos() + self.delay
        }
    }
}

impl<T: FiniteSignal> FiniteSignal for Delay<T> {
    fn len(&self) -> u64 {
        self.inner.len() + self.delay
    }
}

impl<T: SignalReader> SignalReader for Delay<T> {
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let mut buf_len = buf.len();
        let n_channels = self.spec().channels.count() as usize;
        buf_len -= n_channels;

        let mut n_samples = 0;
        if self.rem_padding > 0 {
            n_samples = buf_len.min(self.rem_padding as usize);
            buf[..n_samples].fill(T::Sample::ORIGIN);
            self.rem_padding -= n_samples as u64 / n_channels as u64;

            if n_samples == buf_len {
                return Ok(n_samples);
            }
        }

        n_samples += self.inner.read(&mut buf[n_samples..buf_len])?;
        Ok(n_samples)
    }
}

impl<T: SignalWriter> SignalWriter for Delay<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, PhonicError> {
        if self.rem_padding > 0 {
            let mut null = NullSignal::new(*self.spec());
            self.inner.copy_n(&mut null, self.rem_padding, false)?;
        }

        self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.inner.flush()
    }
}

impl<T: IndexedSignal + SignalSeeker> SignalSeeker for Delay<T> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        todo!()
    }
}

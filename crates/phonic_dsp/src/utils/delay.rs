use phonic_signal::{
    utils::DefaultBuf, FiniteSignal, IndexedSignal, NFrames, PhonicError, PhonicResult, Sample,
    Signal, SignalDuration, SignalReader, SignalSeeker, SignalSpec, SignalWriter,
};
use std::mem::MaybeUninit;

pub struct Delay<T> {
    inner: T,
    delay: u64,
    n_delayed: u64,
}

impl<T: Signal> Delay<T> {
    pub fn new<D: SignalDuration>(inner: T, delay: D) -> Self
    where
        T: IndexedSignal,
    {
        let NFrames { n_frames: delay } = delay.into_duration(inner.spec());
        let n_delayed = if inner.pos() == 0 { 0 } else { delay };

        Self {
            inner,
            delay,
            n_delayed,
        }
    }

    pub fn new_seeked<D: SignalDuration>(inner: T, delay: D) -> Self {
        let NFrames { n_frames } = delay.into_duration(inner.spec());

        Self {
            inner,
            delay: n_frames,
            n_delayed: n_frames,
        }
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
        if self.n_delayed < self.delay {
            self.n_delayed
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

impl<T: SignalReader> Delay<T> {
    fn read_padding(&mut self, buf: &mut [MaybeUninit<T::Sample>]) -> usize {
        let rem_padding = self.delay - self.n_delayed;
        if rem_padding == 0 {
            return 0;
        }

        let mut buf_len = buf.len();
        let n_channels = self.spec().channels.count() as usize;
        buf_len -= buf_len % n_channels;

        let n_padding = buf_len.min(rem_padding as usize * n_channels);
        buf[..n_padding].fill(MaybeUninit::new(T::Sample::ORIGIN));
        self.n_delayed += n_padding as u64 / n_channels as u64;

        n_padding
    }
}

impl<T: SignalReader> SignalReader for Delay<T> {
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let n_padding = self.read_padding(buf);
        if buf.len() - n_padding < self.spec().channels.count() as usize {
            return Ok(n_padding);
        }

        let n_samples = self.inner.read(&mut buf[n_padding..])?;
        Ok(n_padding + n_samples)
    }
}

impl<T: SignalWriter> Delay<T> {
    fn write_padding(&mut self) -> PhonicResult<usize> {
        if self.n_delayed == self.delay {
            return Ok(0);
        }

        let buf = <DefaultBuf<_>>::default();
        let n_channels = self.spec().channels.count() as usize;
        let mut n_written = 0;

        while self.n_delayed < self.delay {
            let rem_interleaved = (self.delay - self.n_delayed) as usize * n_channels;
            let buf_len = buf.len().min(rem_interleaved);
            let n = self.inner.write(&buf[..buf_len])?;

            self.n_delayed += n as u64;
            n_written += n;
        }

        Ok(n_written)
    }
}

impl<T: SignalWriter> SignalWriter for Delay<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        self.write_padding()?;
        self.inner.write(buf)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush()
    }
}

impl<T: IndexedSignal + SignalSeeker> SignalSeeker for Delay<T> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        let pos = self
            .pos()
            .checked_add_signed(offset)
            .ok_or(PhonicError::OutOfBounds)?;

        let inner_pos = pos.saturating_sub(self.delay);
        let inner_offset = inner_pos as i64 - self.inner.pos() as i64;
        self.inner.seek(inner_offset)?;
        self.n_delayed = pos.min(self.delay);

        Ok(())
    }
}

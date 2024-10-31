use crate::{
    FiniteSignal, IndexedSignal, Sample, Signal, SignalReader, SignalSeeker, SignalSpec,
    SignalWriter,
};
use phonic_core::PhonicError;
use std::time::Duration;

pub struct LengthAdapter<T> {
    signal: T,
    len: u64,
    pos: u64,
}

impl<T: IndexedSignal> LengthAdapter<T> {
    pub fn new(signal: T, len: u64) -> Self {
        let pos = signal.pos();
        Self { signal, len, pos }
    }

    pub fn from_interleaved(signal: T, n_samples: u64) -> Self {
        let n_frames = n_samples / signal.spec().channels.count() as u64;
        Self::new(signal, n_frames)
    }

    pub fn from_duration(signal: T, duration: Duration) -> Self {
        let n_frames = (signal.spec().sample_rate as f64 * duration.as_secs_f64()) as u64;
        Self::new(signal, n_frames)
    }
}

impl<T: Signal> Signal for LengthAdapter<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        self.signal.spec()
    }
}

impl<T: Signal> IndexedSignal for LengthAdapter<T> {
    fn pos(&self) -> u64 {
        self.pos
    }
}

impl<T: Signal> FiniteSignal for LengthAdapter<T> {
    fn len(&self) -> u64 {
        self.len
    }
}

impl<T: SignalReader> SignalReader for LengthAdapter<T> {
    fn read(&mut self, _buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let buf_len = _buf.len().min(self.rem_interleaved() as usize);
        let buf = &mut _buf[..buf_len];

        let mut n = self.signal.read(buf)?;
        if n == 0 {
            buf.fill(Self::Sample::ORIGIN);
            n = buf_len
        }

        self.pos += n as u64;
        Ok(n)
    }
}

impl<T: SignalWriter> SignalWriter for LengthAdapter<T> {
    fn write(&mut self, _buf: &[Self::Sample]) -> Result<usize, PhonicError> {
        let buf_len = _buf.len().min(self.rem_interleaved() as usize);
        let buf = &_buf[..buf_len];
        let n = self.signal.write(buf)?;

        self.pos += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.signal.flush()
    }
}

impl<T: SignalSeeker> SignalSeeker for LengthAdapter<T> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        let end_pos = self
            .pos()
            .checked_add_signed(offset)
            .ok_or(PhonicError::OutOfBounds)?;

        self.signal.seek(offset)?;
        self.pos = end_pos;
        Ok(())
    }
}

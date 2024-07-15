use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter};
use phonic_core::PhonicError;
use std::time::Duration;

pub struct DurationAdapter<T: Signal> {
    signal: T,
    rem_frames: Option<u64>,
}

impl<T: Signal> DurationAdapter<T> {
    pub fn new(signal: T, n_frames: Option<u64>) -> Self {
        let mut spec = *signal.spec();

        Self {
            signal,
            rem_frames: n_frames,
        }
    }

    pub fn from_duration(signal: T, duration: Duration) -> Self {
        let n_frames = (signal.spec().frame_rate as f64 * duration.as_secs_f64()) as u64;
        Self::new(signal, Some(n_frames))
    }
}

impl<T: Signal> Signal for DurationAdapter<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        &self.signal.spec()
    }
}

impl<T: SignalReader> SignalReader for DurationAdapter<T> {
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let n_samples = self
            .rem_frames
            .map(|r| (r as usize).min(buf.len()))
            .unwrap_or(buf.len());

        let buf = &mut buf[..n_samples];
        match self.signal.read(buf) {
            Ok(0) => {
                buf.fill(Self::Sample::ORIGIN);
                self.rem_frames.as_mut().map(|r| *r -= n_samples as u64);
                Ok(n_samples)
            }
            Ok(n) => {
                self.rem_frames.as_mut().map(|r| *r -= n as u64);
                Ok(n)
            }
            err => err,
        }
    }
}

impl<T: SignalWriter> SignalWriter for DurationAdapter<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, PhonicError> {
        let n_samples = self
            .rem_frames
            .map(|r| (r as usize).min(buf.len()))
            .unwrap_or(buf.len());

        let result = self.signal.write(&buf[..n_samples]);
        if let Ok(n) = result {
            self.rem_frames.as_mut().map(|r| *r -= n as u64);
        }

        result
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.signal.flush()
    }
}

use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter};
use std::time::Duration;
use syphon_core::SyphonError;

pub struct DurationAdapter<T: Signal> {
    signal: T,
    spec: SignalSpec,
    i: u64,
    inner_consumed: bool,
}

impl<T: Signal> DurationAdapter<T> {
    pub fn new(signal: T, n_frames: Option<u64>) -> Self {
        let mut spec = *signal.spec();
        spec.n_frames = n_frames;

        Self {
            signal,
            spec,
            i: 0,
            inner_consumed: false,
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
        &self.spec
    }
}

impl<T: SignalReader> SignalReader for DurationAdapter<T> {
    fn read(&mut self, buffer: &mut [Self::Sample]) -> Result<usize, SyphonError> {
        if self.spec.n_samples().is_some_and(|n| self.i >= n) {
            return Ok(0);
        }

        let n_samples = self
            .spec
            .n_samples()
            .map(|n| ((n - self.i) as usize).min(buffer.len()))
            .unwrap_or(buffer.len());

        let buffer = &mut buffer[..n_samples];

        if !self.inner_consumed {
            match self.signal.read(buffer) {
                Ok(0) => {
                    self.inner_consumed = true;
                }
                Ok(n) => {
                    self.i += n as u64;
                    return Ok(n);
                }
                err => return err,
            }
        }

        buffer.fill(Self::Sample::ORIGIN);
        self.i += buffer.len() as u64;
        return Ok(buffer.len());
    }
}

impl<T: SignalWriter> SignalWriter for DurationAdapter<T> {
    fn write(&mut self, buffer: &[Self::Sample]) -> Result<usize, SyphonError> {
        if self.spec.n_samples().is_some_and(|n| self.i >= n) {
            return Ok(0);
        }

        let n_samples = self
            .spec
            .n_samples()
            .map(|n| ((n - self.i) as usize).min(buffer.len()))
            .unwrap_or(buffer.len());

        let buffer = &buffer[..n_samples];

        if !self.inner_consumed {
            match self.signal.write(buffer) {
                Ok(0) => {
                    self.inner_consumed = true;
                }
                Ok(n) => {
                    self.i += n as u64;
                    return Ok(n);
                }
                err => return err,
            }
        }

        self.i += buffer.len() as u64;
        return Ok(buffer.len());
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.signal.flush()
    }
}

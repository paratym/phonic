use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};
use std::io::{self, Seek, SeekFrom};

pub struct FrameRateAdapter<T: Signal> {
    signal: T,
    spec: SignalSpec,
}

impl<T: Signal> FrameRateAdapter<T> {
    pub fn new(signal: T, frame_rate: u32) -> Self {
        let mut spec = *signal.spec();
        spec.frame_rate = frame_rate;

        Self { signal, spec }
    }
}

impl<T: Signal> Signal for FrameRateAdapter<T> {
    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: SignalReader<S>, S: Sample> SignalReader<S> for FrameRateAdapter<T> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<T: SignalWriter<S>, S: Sample> SignalWriter<S> for FrameRateAdapter<T> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        todo!()
    }
}

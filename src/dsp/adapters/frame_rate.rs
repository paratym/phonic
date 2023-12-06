use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};
use std::{
    io::{self, Seek, SeekFrom},
};

pub struct FrameRateAdapter<T: Signal<S>, S: Sample> {
    signal: T,
    spec: SignalSpec<S>,
}

impl<T: Signal<S>, S: Sample> FrameRateAdapter<T, S> {
    pub fn new(signal: T, frame_rate: u32) -> Self {
        let mut spec = *signal.spec();
        spec.frame_rate = frame_rate;

        Self { signal, spec }
    }
}

impl<T: Signal<S>, S: Sample> Signal<S> for FrameRateAdapter<T, S> {
    fn spec(&self) -> &SignalSpec<S> {
        &self.spec
    }
}

impl<T: SignalReader<S>, S: Sample> SignalReader<S> for FrameRateAdapter<T, S> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<T: SignalWriter<S>, S: Sample> SignalWriter<S> for FrameRateAdapter<T, S> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<T: Signal<S> + Seek, S: Sample> Seek for FrameRateAdapter<T, S> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        todo!()
    }
}

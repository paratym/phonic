use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};
use std::{
    io::{self, Seek, SeekFrom},
    time::Duration,
};

pub struct NBlocksAdapter<T: Signal<S>, S: Sample> {
    signal: T,
    spec: SignalSpec<S>,
    i: usize,
} 

impl<T: Signal<S>, S: Sample> NBlocksAdapter<T, S> {
    pub fn new(signal: T, n_blocks: u64) -> Self {
        let spec = SignalSpec {
            n_blocks: Some(n_blocks),
            ..*signal.spec()
        };

        Self { signal, spec, i: 0 }
    }

    pub fn from_seconds(signal: T, seconds: f64) -> Self {
        let n_blocks = seconds * signal.spec().block_rate() as f64;
        Self::new(signal, n_blocks as u64)
    }

    pub fn from_duration(signal: T, duration: Duration) -> Self {
        Self::from_seconds(signal, duration.as_secs_f64())
    }
}

impl<T: Signal<S>, S: Sample> Signal<S> for NBlocksAdapter<T, S> {
    fn spec(&self) -> &SignalSpec<S> {
        &self.spec
    }
}

impl<T: SignalReader<S>, S: Sample> SignalReader<S> for NBlocksAdapter<T, S> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<T: SignalWriter<S>, S: Sample> SignalWriter<S> for NBlocksAdapter<T, S> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<T: Signal<S> + Seek, S: Sample> Seek for NBlocksAdapter<T, S> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        todo!()
    }
}

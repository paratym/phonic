use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};
use std::{
    io::{self, Seek, SeekFrom},
    marker::PhantomData,
};

pub struct NBlocksAdapter<T: Signal, S: Sample> {
    signal: T,
    spec: SignalSpec,
    i: usize,
    _sample_type: PhantomData<S>,
}

impl<T: Signal, S: Sample> NBlocksAdapter<T, S> {
    pub fn from_signal(signal: T, n_blocks: u64) -> Self {
        let spec = SignalSpec {
            n_blocks: Some(n_blocks),
            ..*signal.spec()
        };

        Self {
            signal,
            spec,
            i: 0,
            _sample_type: PhantomData,
        }
    }
}

impl<T: Signal, S: Sample> Signal for NBlocksAdapter<T, S> {
    fn spec(&self) -> &SignalSpec {
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

impl<T: Signal + Seek, S: Sample> Seek for NBlocksAdapter<T, S> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        todo!()
    }
}

use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};
use std::io::{self, Seek, SeekFrom};

pub struct BlockSizeAdapter<T: Signal, S: Sample> {
    signal: T,
    spec: SignalSpec,
    buffer: Box<[S]>,
    inner_block_size: usize,
    n_buffered: usize,
    n_read: usize,
}

impl<T: Signal, S: Sample> BlockSizeAdapter<T, S> {
    pub fn from_signal(signal: T, block_size: usize) -> Self {
        let mut spec = *signal.spec();
        let inner_block_size = spec.block_size;
        spec.block_size = block_size;

        todo!()
    }
}

impl<T: Signal, S: Sample> Signal for BlockSizeAdapter<T, S> {
    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: SignalReader<S>, S: Sample> SignalReader<S> for BlockSizeAdapter<T, S> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<T: SignalWriter<S>, S: Sample> SignalWriter<S> for BlockSizeAdapter<T, S> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.signal.flush()
    }
}

impl<T: Signal + Seek, S: Sample> Seek for BlockSizeAdapter<T, S> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        todo!()
    }
}

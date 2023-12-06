use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};
use std::{
    io::{self, Seek, SeekFrom},
    time::Duration,
};

pub struct NBlocksAdapter<T: Signal<S>, S: Sample> {
    signal: T,
    spec: SignalSpec<S>,
    n_blocks: u64,
    i: u64,
    inner_consumed: bool
} 

impl<T: Signal<S>, S: Sample> NBlocksAdapter<T, S> {
    pub fn new(signal: T, n_blocks: u64) -> Self {
        let mut spec = *signal.spec();
        spec.n_blocks = Some(n_blocks);

        Self {
            signal,
            spec,
            n_blocks,
            i: 0,
            inner_consumed: false
        }
    }

    pub fn from_seconds(signal: T, seconds: f64) -> Self {
        let n_blocks = seconds * signal.spec().block_rate();
        Self::new(signal, n_blocks as u64)
    }

    pub fn from_duration(signal: T, duration: Duration) -> Self {
        todo!()
    }
}

impl<T: Signal<S>, S: Sample> Signal<S> for NBlocksAdapter<T, S> {
    fn spec(&self) -> &SignalSpec<S> {
        &self.spec
    }
}

impl<T: SignalReader<S>, S: Sample> SignalReader<S> for NBlocksAdapter<T, S> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        if self.i >= self.n_blocks {
            return Ok(0);
        }

        let samples_per_block = self.spec.samples_per_block();
        let n_blocks = (buffer.len() / samples_per_block).min((self.n_blocks - self.i) as usize);
        let buffer = &mut buffer[..n_blocks * samples_per_block];

        if !self.inner_consumed {
            match self.signal.read(buffer) {
                Ok(0) => {
                    self.inner_consumed = true;
                },
                Ok(n) => {
                    self.i += n as u64;
                    return Ok(n);
                },
                err => return err 
            }
        }

        buffer.fill(S::ORIGIN);

        self.i += n_blocks as u64;
        return Ok(n_blocks);
    }
}

impl<T: SignalWriter<S>, S: Sample> SignalWriter<S> for NBlocksAdapter<T, S> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        if self.i >= self.n_blocks {
            return Ok(0);
        }

        let samples_per_block = self.spec.samples_per_block();
        let n_blocks = (buffer.len() / samples_per_block).min((self.n_blocks - self.i) as usize);
        let buffer = &buffer[..n_blocks * samples_per_block];

        if !self.inner_consumed {
            match self.signal.write(buffer) {
                Ok(0) => {
                    self.inner_consumed = true;
                },
                Ok(n) => {
                    self.i += n as u64;
                    return Ok(n);
                },
                err => return err 
            }
        }
        
        self.i += n_blocks as u64;
        return Ok(n_blocks);
    }
}

impl<T: Signal<S> + Seek, S: Sample> Seek for NBlocksAdapter<T, S> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        todo!()
    }
}

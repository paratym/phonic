use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};
use std::io::{Seek, SeekFrom};

pub trait Pipe<S: Sample> {
    fn spec(&self) -> &SignalSpec;
    fn process(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;
}

pub struct Pipeline<S: Sample, T: Signal> {
    signal: T,
    pipes: Box<[Box<dyn Pipe<S>>]>,
}

pub struct PipelineBuilder<S: Sample, T: Signal> {
    signal: T,
    pipes: Vec<Box<dyn Pipe<S>>>,
}

impl<S: Sample, T: Signal> Pipeline<S, T> {
    pub fn builder(signal: T) -> PipelineBuilder<S, T> {
        PipelineBuilder::from_signal(signal)
    }
}

impl<S: Sample, T: Signal> Signal for Pipeline<S, T> {
    fn spec(&self) -> &SignalSpec {
        match self.pipes.last() {
            Some(pipe) => pipe.spec(),
            None => self.signal.spec(),
        }
    }
}

impl<S: Sample, T: SignalReader<S>> SignalReader<S> for Pipeline<S, T> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        let n_read = self.signal.read(buffer)?;
        for pipe in self.pipes.as_mut() {
            pipe.process(&mut buffer[..n_read])?;
        }

        Ok(n_read)
    }
}

impl<S: Sample, T: SignalWriter<S>> SignalWriter<S> for Pipeline<S, T> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        self.signal.write(buffer)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.signal.flush()
    }
}

impl<S: Sample, T: Signal + Seek> Seek for Pipeline<S, T> {
    fn seek(&mut self, offset: SeekFrom) -> std::io::Result<u64> {
        self.signal.seek(offset)
    }
}

impl<S: Sample, T: Signal> PipelineBuilder<S, T> {
    pub fn from_signal(signal: T) -> Self {
        Self {
            signal,
            pipes: vec![],
        }
    }

    pub fn pipe(mut self, pipe: impl Pipe<S> + 'static) -> Self {
        self.pipes.push(Box::new(pipe));
        self
    }

    pub fn build(self) -> Result<Pipeline<S, T>, SyphonError> {
        let spec = self.signal.spec();
        if self.pipes.iter().all(|pipe| pipe.spec() == spec) {
            return Err(SyphonError::SignalMismatch);
        }

        Ok(Pipeline {
            signal: self.signal,
            pipes: self.pipes.into_boxed_slice(),
        })
    }
}

use crate::core::{Sample, SignalSpec, Source, SyphonError};

pub trait Pipe<S: Sample> {
    fn signal_spec(&self) -> SignalSpec;
    fn process(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;
}

pub struct Pipeline<S: Sample> {
  source: Box<dyn Source<S>>,
  pipes: Vec<Box<dyn Pipe<S>>>,
}

impl<S: Sample> Pipeline<S> {
    pub fn from_source(source: impl Source<S> + 'static) -> Self {
        Self {
            source: Box::new(source),
            pipes: vec![],
        }
    }

    pub fn pipe(mut self, pipe: impl Pipe<S> + 'static) -> Self {
        self.pipes.push(Box::new(pipe));
        self
    }
}

impl<S: Sample> Source<S> for Pipeline<S> {
    fn signal_spec(&self) -> SignalSpec {
        match self.pipes.last() {
            Some(pipe) => pipe.signal_spec(),
            None => self.source.signal_spec(),
        }
    }

    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        let n_read = self.source.read(buffer)?;
        for pipe in &mut self.pipes {
            pipe.process(&mut buffer[..n_read])?;
        }
        
        Ok(n_read)
    }
}
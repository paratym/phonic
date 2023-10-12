use crate::core::{Sample, SampleReader, SignalSpec, SyphonError};

pub trait Pipe<S: Sample> {
    fn signal_spec(&self) -> SignalSpec;
    fn process(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;
}

pub struct Pipeline<S: Sample> {
    source: Box<dyn SampleReader<S>>,
    pipes: Box<[Box<dyn Pipe<S>>]>,
}

impl<S: Sample> SampleReader<S> for Pipeline<S> {
    fn signal_spec(&self) -> SignalSpec {
        match self.pipes.last() {
            Some(pipe) => pipe.signal_spec(),
            None => self.source.signal_spec(),
        }
    }

    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        let n_read = self.source.read(buffer)?;
        for pipe in self.pipes.as_mut() {
            pipe.process(&mut buffer[..n_read])?;
        }

        Ok(n_read)
    }
}

pub struct PipelineBuilder<S: Sample> {
    source: Box<dyn SampleReader<S>>,
    pipes: Vec<Box<dyn Pipe<S>>>,
}

impl<S: Sample> PipelineBuilder<S> {
    pub fn from_source(source: impl SampleReader<S> + 'static) -> Self {
        Self {
            source: Box::new(source),
            pipes: vec![],
        }
    }

    pub fn pipe(mut self, pipe: impl Pipe<S> + 'static) -> Self {
        self.pipes.push(Box::new(pipe));
        self
    }

    pub fn build(self) -> Pipeline<S> {
        Pipeline {
            source: self.source,
            pipes: self.pipes.into_boxed_slice(),
        }
    }
}

use crate::{
    io::{SampleReader, StreamSpec},
    Sample, SyphonError,
};

pub trait Pipe<S: Sample> {
    fn stream_spec(&self) -> &StreamSpec;
    fn process(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;
}

pub struct Pipeline<S: Sample> {
    source: Box<dyn SampleReader<S>>,
    pipes: Box<[Box<dyn Pipe<S>>]>,
}

impl<S: Sample> Pipeline<S> {
    fn from_source(source: impl SampleReader<S> + 'static) -> PipelineBuilder<S> {
        PipelineBuilder::from_source(source)
    }
}

impl<S: Sample> SampleReader<S> for Pipeline<S> {
    fn stream_spec(&self) -> &StreamSpec {
        match self.pipes.last() {
            Some(pipe) => pipe.stream_spec(),
            None => self.source.stream_spec(),
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

    pub fn try_build(self) -> Result<Pipeline<S>, SyphonError> {
        let stream_spec = self.source.stream_spec();
        if self
            .pipes
            .iter()
            .all(|pipe| pipe.stream_spec() == stream_spec)
        {
            return Err(SyphonError::StreamMismatch);
        }

        Ok(Pipeline {
            source: self.source,
            pipes: self.pipes.into_boxed_slice(),
        })
    }
}

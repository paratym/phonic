use crate::{Sample, SampleReader, SampleStream, SampleWriter, StreamSpec, SyphonError};
use std::io::{Seek, SeekFrom};

pub trait Pipe<S: Sample> {
    fn spec(&self) -> &StreamSpec;
    fn process(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;
}

pub struct Pipeline<S: Sample, T: SampleStream<S>> {
    stream: T,
    pipes: Box<[Box<dyn Pipe<S>>]>,
}

pub struct PipelineBuilder<S: Sample, T: SampleStream<S>> {
    stream: T,
    pipes: Vec<Box<dyn Pipe<S>>>,
}

impl<S: Sample, T: SampleStream<S>> Pipeline<S, T> {
    pub fn from_stream(stream: T) -> PipelineBuilder<S, T> {
        PipelineBuilder::from_stream(stream)
    }
}

impl<S: Sample, T: SampleStream<S>> SampleStream<S> for Pipeline<S, T> {
    fn spec(&self) -> &StreamSpec {
        match self.pipes.last() {
            Some(pipe) => pipe.spec(),
            None => self.stream.spec(),
        }
    }
}

impl<S: Sample, T: SampleReader<S>> SampleReader<S> for Pipeline<S, T> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        let n_read = self.stream.read(buffer)?;
        for pipe in self.pipes.as_mut() {
            pipe.process(&mut buffer[..n_read])?;
        }

        Ok(n_read)
    }
}

impl<S: Sample, T: SampleWriter<S>> SampleWriter<S> for Pipeline<S, T> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        self.stream.write(buffer)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.stream.flush()
    }
}

impl<S: Sample, T: SampleStream<S> + Seek> Seek for Pipeline<S, T> {
    fn seek(&mut self, offset: SeekFrom) -> std::io::Result<u64> {
        self.stream.seek(offset)
    }
}

impl<S: Sample, T: SampleStream<S>> PipelineBuilder<S, T> {
    pub fn from_stream(stream: T) -> Self {
        Self {
            stream,
            pipes: vec![],
        }
    }

    pub fn pipe(mut self, pipe: impl Pipe<S> + 'static) -> Self {
        self.pipes.push(Box::new(pipe));
        self
    }

    pub fn build(self) -> Result<Pipeline<S, T>, SyphonError> {
        let stream_spec = self.stream.spec();
        if self.pipes.iter().all(|pipe| pipe.spec() == stream_spec) {
            return Err(SyphonError::StreamMismatch);
        }

        Ok(Pipeline {
            stream: self.stream,
            pipes: self.pipes.into_boxed_slice(),
        })
    }
}

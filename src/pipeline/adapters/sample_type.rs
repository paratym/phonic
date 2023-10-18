use crate::{
    io::{SampleReader, StreamSpec},
    FromSample, Sample, SyphonError,
};
use std::marker::PhantomData;

pub struct SampleTypeAdapter<S: Sample, O: Sample + FromSample<S>> {
    source: Box<dyn SampleReader<S>>,
    buffer: Box<[S]>,
    stream_spec: StreamSpec,
    _sample_type: PhantomData<O>,
}

impl<S: Sample, O: Sample + FromSample<S>> SampleTypeAdapter<S, O> {
    pub fn new(source: Box<dyn SampleReader<S>>) -> Self {
        let stream_spec = StreamSpec {
            sample_format: O::FORMAT,
            ..*source.stream_spec()
        };

        Self {
            source,
            buffer: vec![S::MID; stream_spec.block_size].into_boxed_slice(),
            stream_spec,
            _sample_type: PhantomData,
        }
    }
}

impl<S: Sample, O: Sample + FromSample<S>> SampleReader<O> for SampleTypeAdapter<S, O> {
    fn stream_spec(&self) -> &StreamSpec {
        &self.stream_spec
    }

    fn read(&mut self, buffer: &mut [O]) -> Result<usize, SyphonError> {
        let n_read = match self.source.read(&mut self.buffer) {
            Ok(0) => return Ok(0),
            Ok(n) if n % self.stream_spec.block_size == 0 => n,
            Ok(_) => return Err(SyphonError::MalformedData),
            Err(err) => return Err(err),
        };

        for (in_sample, out_sample) in self.buffer.iter().zip(buffer.iter_mut()) {
            *out_sample = O::from(*in_sample);
        }

        Ok(n_read)
    }
}

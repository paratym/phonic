use std::marker::PhantomData;
use crate::{Sample, FromSample, io::{SampleReader, SignalSpec}, SyphonError};

pub struct SampleTypeAdapter<S: Sample, O: Sample + FromSample<S>> {
    source: Box<dyn SampleReader<S>>,
    buffer: Box<[S]>,
    signal_spec: SignalSpec,
    _sample_type: PhantomData<O>,
}

impl<S: Sample, O: Sample + FromSample<S>> SampleTypeAdapter<S, O> {
    pub fn new(source: Box<dyn SampleReader<S>>) -> Self {
        let signal_spec = SignalSpec {
            sample_format: O::FORMAT,
            bytes_per_sample: O::N_BYTES as u16,
            ..*source.signal_spec()
        };

        Self {
            source,
            buffer: vec![S::MID; signal_spec.block_size].into_boxed_slice(),
            signal_spec,
            _sample_type: PhantomData,
        }
    }
}

impl<S: Sample, O: Sample + FromSample<S>> SampleReader<O> for SampleTypeAdapter<S, O> {
    fn signal_spec(&self) -> &SignalSpec {
        &self.signal_spec
    }

    fn read(&mut self, buffer: &mut [O]) -> Result<usize, SyphonError> {
        let n_read = match self.source.read(&mut self.buffer) {
            Ok(0) => return Ok(0),
            Ok(n) if n % self.signal_spec.block_size == 0 => n,
            Ok(_) => return Err(SyphonError::MalformedData),
            Err(err) => return Err(err),
        };

        for (in_sample, out_sample) in self.buffer.iter().zip(buffer.iter_mut()) {
            *out_sample = O::from(*in_sample);
        }

        Ok(n_read)
    }
}

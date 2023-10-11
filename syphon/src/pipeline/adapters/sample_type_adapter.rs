use std::marker::PhantomData;
use crate::core::{SignalSpec, SampleReader, Sample, FromSample, SyphonError};

pub struct SampleTypeAdapter<S: Sample, O: Sample + FromSample<S>>{
    signal_spec: SignalSpec,
    source: Box<dyn SampleReader<S>>,
    buffer: Box<[S]>,
    out_sample: PhantomData<O>
}

impl<S: Sample, O: Sample + FromSample<S>> SampleTypeAdapter<S, O> {
    pub fn new(source: impl SampleReader<S> + 'static) -> Self {
        let signal_spec = SignalSpec {
            sample_format: O::FORMAT,
            bits_per_sample: (O::N_BYTES * 8) as u16,
            ..source.signal_spec()
        };

        Self {
            signal_spec,
            source: Box::new(source),
            buffer: vec![S::MID; signal_spec.block_size as usize].into_boxed_slice(),
            out_sample: PhantomData,
        }
    }
}

impl<S: Sample, O: Sample + FromSample<S>> SampleReader<O> for SampleTypeAdapter<S, O> {
    fn signal_spec(&self) -> SignalSpec {
        self.signal_spec
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
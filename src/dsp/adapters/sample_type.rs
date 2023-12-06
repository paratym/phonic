use crate::{IntoSample, Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};
use std::io::{self, Seek, SeekFrom};

pub struct SampleTypeAdapter<T: Signal<S>, S: Sample, O: Sample> {
    signal: T,
    buffer: Box<[S]>,
    spec: SignalSpec<O>,
}

impl<T: Signal<S>, S: Sample, O: Sample> SampleTypeAdapter<T, S, O> {
    pub fn new(signal: T) -> Self {
        let spec = signal.spec().cast_sample_type(O::ORIGIN);
        let buffer = vec![S::ORIGIN; spec.samples_per_block()].into_boxed_slice();

        Self {
            signal,
            buffer,
            spec,
        }
    }
}

impl<T: Signal<S>, S: Sample, O: Sample> Signal<O> for SampleTypeAdapter<T, S, O> {
    fn spec(&self) -> &SignalSpec<O> {
        &self.spec
    }
}

impl<T, S, O> SignalReader<O> for SampleTypeAdapter<T, S, O>
where
    T: SignalReader<S>,
    S: Sample + IntoSample<O>,
    O: Sample,
{
    fn read(&mut self, buffer: &mut [O]) -> Result<usize, SyphonError> {
        let buf_len = buffer.len().min(self.buffer.len());
        let n_blocks = self.signal.read(&mut self.buffer[..buf_len])?;
        let n_samples = n_blocks * self.signal.spec().samples_per_block();

        for (inner, outer) in self.buffer.iter().zip(buffer[..n_samples].iter_mut()) {
            *outer = inner.into_sample();
        }

        Ok(n_blocks)
    }
}

impl<T, S, O> SignalWriter<O> for SampleTypeAdapter<T, S, O>
where
    T: SignalWriter<S>,
    S: Sample,
    O: Sample + IntoSample<S>,
{
    fn write(&mut self, buffer: &[O]) -> Result<usize, SyphonError> {
        let buf_len = buffer.len().min(self.buffer.len());

        for (outer, inner) in buffer[..buf_len].iter().zip(self.buffer.iter_mut()) {
            *inner = outer.into_sample();
        }

        self.signal.write(&self.buffer[..buf_len])
    }
}

impl<T: Signal<S> + Seek, S: Sample, O: Sample> Seek for SampleTypeAdapter<T, S, O> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        self.signal.seek(offset)
    }
}

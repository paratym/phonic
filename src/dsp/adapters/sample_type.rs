use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError, IntoSample};
use std::{
    io::{self, Seek, SeekFrom},
    marker::PhantomData,
};

pub struct SampleTypeAdapter<T: Signal, S: Sample, O: Sample> {
    signal: T,
    buffer: Box<[S]>,
    spec: SignalSpec,
    _sample_type: PhantomData<O>,
}

impl<T: Signal, S: Sample, O: Sample> SampleTypeAdapter<T, S, O> {
    pub fn from_signal(signal: T) -> Self {
        let spec = SignalSpec {
            sample_format: O::FORMAT,
            ..*signal.spec()
        };

        let buffer = vec![S::MID; spec.samples_per_block()].into_boxed_slice();

        Self {
            signal,
            buffer,
            spec,
            _sample_type: PhantomData,
        }
    }
}

impl<T: Signal, S: Sample, O: Sample> Signal for SampleTypeAdapter<T, S, O> {
    fn spec(&self) -> &SignalSpec {
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
        let n_read = self.signal.read(&mut self.buffer[..buf_len])?;

        for (inner, outer) in self.buffer.iter().zip(buffer[..n_read].iter_mut()) {
            *outer = inner.into_sample();
        }

        Ok(n_read)
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

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.signal.flush()
    }
}

impl<T: Signal + Seek, S: Sample, O: Sample> Seek for SampleTypeAdapter<T, S, O> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        self.signal.seek(offset)
    }
}

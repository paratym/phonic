use crate::{IntoSample, Sample, Signal, SignalReader, SignalSpec, SignalWriter};
use std::marker::PhantomData;
use syphon_core::SyphonError;

pub struct SampleTypeAdapter<T: Signal, S: Sample> {
    signal: T,
    buffer: Box<[T::Sample]>,
    _sample: PhantomData<S>,
}

impl<T: Signal, S: Sample> SampleTypeAdapter<T, S> {
    pub fn new(signal: T) -> Self {
        let buf_len = signal.spec().channels.count() as usize;
        let buffer = vec![T::Sample::ORIGIN; buf_len].into_boxed_slice();
        Self {
            signal,
            buffer,
            _sample: PhantomData,
        }
    }
}

impl<T: Signal, S: Sample> Signal for SampleTypeAdapter<T, S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        self.signal.spec()
    }
}

impl<T, S> SignalReader for SampleTypeAdapter<T, S>
where
    T: SignalReader,
    T::Sample: IntoSample<S>,
    S: Sample,
{
    fn read(&mut self, buffer: &mut [Self::Sample]) -> Result<usize, SyphonError> {
        let buf_len = buffer.len().min(self.buffer.len());
        let n = self.signal.read(&mut self.buffer[..buf_len])?;

        for (inner, outer) in self.buffer.iter().zip(buffer[..n].iter_mut()) {
            *outer = inner.into_sample();
        }

        Ok(n)
    }
}

impl<T, S> SignalWriter for SampleTypeAdapter<T, S>
where
    T: SignalWriter,
    S: Sample,
    Self::Sample: IntoSample<T::Sample>,
{
    fn write(&mut self, buffer: &[Self::Sample]) -> Result<usize, SyphonError> {
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

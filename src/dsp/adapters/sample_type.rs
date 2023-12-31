use crate::{
    io::{DynSignalReader, DynSignalWriter},
    FromKnownSample, IntoKnownSample, IntoSample, Sample, Signal, SignalReader, SignalSpec,
    SignalWriter, SyphonError,
};

pub struct SampleTypeAdapter<S: Sample, T: Signal> {
    signal: T,
    buffer: Box<[S]>,
}

impl<S: Sample, T: Signal> SampleTypeAdapter<S, T> {
    pub fn new(signal: T) -> Self {
        let buf_len = signal.spec().channels.count() as usize;
        let buffer = vec![S::ORIGIN; buf_len].into_boxed_slice();
        Self { signal, buffer }
    }
}

impl<S: Sample, T: Signal> Signal for SampleTypeAdapter<S, T> {
    fn spec(&self) -> &SignalSpec {
        self.signal.spec()
    }
}

impl<S, O, T> SignalReader<O> for SampleTypeAdapter<S, T>
where
    S: Sample + IntoSample<O>,
    O: Sample,
    T: SignalReader<S>,
{
    fn read(&mut self, buffer: &mut [O]) -> Result<usize, SyphonError> {
        let buf_len = buffer.len().min(self.buffer.len());
        let n = self.signal.read(&mut self.buffer[..buf_len])?;

        for (inner, outer) in self.buffer.iter().zip(buffer[..n].iter_mut()) {
            *outer = inner.into_sample();
        }

        Ok(n)
    }
}

impl<S: Sample + IntoKnownSample, T: SignalReader<S>> DynSignalReader for SampleTypeAdapter<S, T> {}

impl<S, O, T> SignalWriter<O> for SampleTypeAdapter<S, T>
where
    S: Sample,
    O: Sample + IntoSample<S>,
    T: SignalWriter<S>,
{
    fn write(&mut self, buffer: &[O]) -> Result<usize, SyphonError> {
        let buf_len = buffer.len().min(self.buffer.len());

        for (outer, inner) in buffer[..buf_len].iter().zip(self.buffer.iter_mut()) {
            *inner = outer.into_sample();
        }

        self.signal.write(&self.buffer[..buf_len])
    }
}

impl<S: Sample + FromKnownSample, T: SignalWriter<S>> DynSignalWriter for SampleTypeAdapter<S, T> {}

use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};
use std::{
    io::{self, Seek, SeekFrom},
    marker::PhantomData,
};

pub struct SampleRateAdapter<T: Signal, S: Sample> {
    signal: T,
    spec: SignalSpec,
    _sample_type: PhantomData<S>,
}

impl<T: Signal, S: Sample> SampleRateAdapter<T, S> {
    pub fn from_signal(signal: T, sample_rate: u32) -> Self {
        let spec = SignalSpec {
            sample_rate,
            ..*signal.spec()
        };

        Self {
            signal,
            spec,
            _sample_type: PhantomData,
        }
    }
}

impl<T: Signal, S: Sample> Signal for SampleRateAdapter<T, S> {
    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: SignalReader<S>, S: Sample> SignalReader<S> for SampleRateAdapter<T, S> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<T: SignalWriter<S>, S: Sample> SignalWriter<S> for SampleRateAdapter<T, S> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<T: Signal + Seek, S: Sample> Seek for SampleRateAdapter<T, S> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        todo!()
    }
}

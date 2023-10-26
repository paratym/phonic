use crate::{Sample, Signal, SignalSpec, SignalReader, SyphonError, SignalWriter};
use std::{marker::PhantomData, io::{Seek, SeekFrom, self}};

pub struct ChannelsAdapter<T: Signal, S: Sample> {
    signal: T,
    spec: SignalSpec,
    _sample_type: PhantomData<S>,
}

impl<T: Signal, S: Sample> ChannelsAdapter<T, S> {
    pub fn from_signal(signal: T, n_channels: u8) -> Self {
        let spec = SignalSpec {
            n_channels,
            ..*signal.spec()
        };

        Self {
            signal,
            spec,
            _sample_type: PhantomData,
        }
    }
}

impl<T: Signal, S: Sample> Signal for ChannelsAdapter<T, S> {
    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: SignalReader<S>, S: Sample> SignalReader<S> for ChannelsAdapter<T, S> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<T: SignalWriter<S>, S: Sample> SignalWriter<S> for ChannelsAdapter<T, S> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.signal.flush()
    }
}

impl<T: Signal + Seek, S: Sample> Seek for ChannelsAdapter<T, S> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        todo!()
    }
}

use crate::{Channels, Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};
use std::io::{self, Seek, SeekFrom};

pub struct ChannelsAdapter<T: Signal> {
    signal: T,
    spec: SignalSpec,
}

impl<T: Signal> ChannelsAdapter<T> {
    pub fn new(signal: T, channels: Channels) -> Self {
        let mut spec = *signal.spec();
        spec.channels = channels;

        Self { signal, spec }
    }
}

impl<T: Signal> Signal for ChannelsAdapter<T> {
    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<S: Sample, T: SignalReader<S>> SignalReader<S> for ChannelsAdapter<T> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<S: Sample, T: SignalWriter<S>> SignalWriter<S> for ChannelsAdapter<T> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        todo!()
    }
}

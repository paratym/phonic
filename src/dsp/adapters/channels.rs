use crate::{Channels, Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};
use std::io::{self, Seek, SeekFrom};

pub struct ChannelsAdapter<T: Signal<S>, S: Sample> {
    signal: T,
    spec: SignalSpec<S>,
}

impl<T: Signal<S>, S: Sample> ChannelsAdapter<T, S> {
    pub fn new(signal: T, channels: Channels) -> Self {
        let mut spec = *signal.spec();
        spec.channels = channels;

        Self { signal, spec }
    }
}

impl<T: Signal<S>, S: Sample> Signal<S> for ChannelsAdapter<T, S> {
    fn spec(&self) -> &SignalSpec<S> {
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
}

impl<T: Signal<S> + Seek, S: Sample> Seek for ChannelsAdapter<T, S> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, io::Error> {
        todo!()
    }
}

use crate::{
    Channels, FiniteSignal, IndexedSignal, Signal, SignalReader, SignalSeeker, SignalSpec,
    SignalWriter,
};
use phonic_core::PhonicError;

pub struct ChannelsAdapter<T: Signal> {
    signal: T,
    spec: SignalSpec,
}

impl<T: Signal> ChannelsAdapter<T> {
    pub fn new(signal: T, channels: impl Into<Channels>) -> Self {
        let mut spec = *signal.spec();
        spec.channels = channels.into();

        Self { signal, spec }
    }
}

impl<T: Signal> Signal for ChannelsAdapter<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: IndexedSignal> IndexedSignal for ChannelsAdapter<T> {
    fn pos(&self) -> u64 {
        self.signal.pos()
    }
}

impl<T: FiniteSignal> FiniteSignal for ChannelsAdapter<T> {
    fn len(&self) -> u64 {
        self.signal.len()
    }
}

impl<T: SignalReader> SignalReader for ChannelsAdapter<T> {
    fn read(&mut self, buffer: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        todo!()
    }
}

impl<T: SignalWriter> SignalWriter for ChannelsAdapter<T> {
    fn write(&mut self, buffer: &[Self::Sample]) -> Result<usize, PhonicError> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        todo!()
    }
}

impl<T: SignalSeeker> SignalSeeker for ChannelsAdapter<T> {
    fn seek(&mut self, frame_offset: i64) -> Result<(), PhonicError> {
        self.signal.seek(frame_offset)
    }
}

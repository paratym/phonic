use crate::{Signal, SignalReader, SignalSpec, SignalWriter};
use phonic_core::PhonicError;

pub struct FrameRateAdapter<T: Signal> {
    signal: T,
    spec: SignalSpec,
}

impl<T: Signal> FrameRateAdapter<T> {
    pub fn new(signal: T, frame_rate: u32) -> Self {
        let mut spec = *signal.spec();
        spec.frame_rate = frame_rate;

        Self { signal, spec }
    }
}

impl<T: Signal> Signal for FrameRateAdapter<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: Signal> SignalReader for FrameRateAdapter<T> {
    fn read(&mut self, buffer: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        todo!()
    }
}

impl<T: Signal> SignalWriter for FrameRateAdapter<T> {
    fn write(&mut self, buffer: &[Self::Sample]) -> Result<usize, PhonicError> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        todo!()
    }
}

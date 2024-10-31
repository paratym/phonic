use crate::{
    FiniteSignal, IndexedSignal, Signal, SignalReader, SignalSeeker, SignalSpec, SignalWriter,
};
use phonic_core::PhonicError;

pub struct SampleRateAdapter<T: Signal> {
    signal: T,
    spec: SignalSpec,
}

impl<T: Signal> SampleRateAdapter<T> {
    pub fn new(signal: T, sample_rate: u32) -> Self {
        let mut spec = *signal.spec();
        spec.sample_rate = sample_rate;

        Self { signal, spec }
    }
}

impl<T: Signal> Signal for SampleRateAdapter<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: IndexedSignal> IndexedSignal for SampleRateAdapter<T> {
    fn pos(&self) -> u64 {
        todo!()
    }
}

impl<T: FiniteSignal> FiniteSignal for SampleRateAdapter<T> {
    fn len(&self) -> u64 {
        todo!()
    }
}

impl<T: SignalReader> SignalReader for SampleRateAdapter<T> {
    fn read(&mut self, buffer: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        todo!()
    }
}

impl<T: SignalWriter> SignalWriter for SampleRateAdapter<T> {
    fn write(&mut self, buffer: &[Self::Sample]) -> Result<usize, PhonicError> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        todo!()
    }
}

impl<T: SignalSeeker> SignalSeeker for SampleRateAdapter<T> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        todo!()
    }
}

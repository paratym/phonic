use phonic_core::PhonicError;
use phonic_signal::{Sample, Signal, SignalReader, SignalSpec, SignalWriter};
use std::marker::PhantomData;

pub struct NullSignal<S> {
    spec: SignalSpec,
    _sample: PhantomData<S>,
}

impl<S> NullSignal<S> {
    pub fn new(spec: SignalSpec) -> Self {
        Self {
            spec,
            _sample: PhantomData,
        }
    }
}

impl<S: Sample> Signal for NullSignal<S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<S: Sample> SignalReader for NullSignal<S> {
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let mut len = buf.len();
        len -= len % self.spec().channels.count() as usize;

        buf[..len].fill(Self::Sample::ORIGIN);
        Ok(len)
    }
}

impl<S: Sample> SignalWriter for NullSignal<S> {
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, PhonicError> {
        let mut len = buf.len();
        len -= len % self.spec().channels.count() as usize;

        Ok(len)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        Ok(())
    }
}
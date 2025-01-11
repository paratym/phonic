use crate::{PhonicResult, Sample, Signal, SignalReader, SignalSpec, SignalWriter};
use std::{marker::PhantomData, mem::MaybeUninit};

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
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let mut len = buf.len();
        len -= len % self.spec().channels.count() as usize;

        buf[..len].fill(MaybeUninit::new(Self::Sample::ORIGIN));
        Ok(len)
    }
}

impl<S: Sample> SignalWriter for NullSignal<S> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let mut len = buf.len();
        len -= len % self.spec().channels.count() as usize;

        Ok(len)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Ok(())
    }
}

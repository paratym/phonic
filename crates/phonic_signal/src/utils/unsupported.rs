use crate::{
    delegate_signal, FiniteSignal, PhonicError, PhonicResult, Signal, SignalReader, SignalSeeker,
    SignalWriter,
};
use std::mem::MaybeUninit;

pub struct Infinite<T>(pub T);
pub struct UnReadable<T>(pub T);
pub struct UnWriteable<T>(pub T);
pub struct UnSeekable<T>(pub T);

delegate_signal! {
    delegate<T> * + !FiniteSignal for Infinite<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Signal> FiniteSignal for Infinite<T> {
    fn len(&self) -> u64 {
        u64::MAX
    }
}

delegate_signal! {
    delegate<T> * + !SignalReader for UnReadable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Signal> SignalReader for UnReadable<T> {
    fn read(&mut self, _buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        Err(PhonicError::Unsupported)
    }
}

delegate_signal! {
    delegate<T> * + !SignalWriter for UnWriteable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Signal> SignalWriter for UnWriteable<T> {
    fn write(&mut self, _buf: &[Self::Sample]) -> PhonicResult<usize> {
        Err(PhonicError::Unsupported)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Err(PhonicError::Unsupported)
    }
}

delegate_signal! {
    delegate<T> * + !SignalSeeker for UnSeekable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Signal> SignalSeeker for UnSeekable<T> {
    fn seek(&mut self, _offset: i64) -> PhonicResult<()> {
        Err(PhonicError::Unsupported)
    }
}

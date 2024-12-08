use crate::{
    FiniteSignal, PhonicError, PhonicResult, Signal, SignalReader, SignalSeeker, SignalWriter,
};
use phonic_macro::impl_deref_signal;
use std::mem::MaybeUninit;

pub struct Infinite<T>(pub T);
pub struct UnReadable<T>(pub T);
pub struct UnWriteable<T>(pub T);
pub struct UnSeekable<T>(pub T);

impl_deref_signal! {
    impl<T> _ + !FiniteSignal for Infinite<T> {
        type Target = T;

        &self -> &self.0;
        &mut self -> &mut self.0;
    }
}

impl<T: Signal> FiniteSignal for Infinite<T> {
    fn len(&self) -> u64 {
        u64::MAX
    }
}

impl_deref_signal! {
    impl<T> _ + !SignalReader for UnReadable<T> {
        type Target = T;

        &self -> &self.0;
        &mut self -> &mut self.0;
    }
}

impl<T: Signal> SignalReader for UnReadable<T> {
    fn read(&mut self, _buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        Err(PhonicError::Unsupported)
    }
}

impl_deref_signal! {
    impl<T> _ + !SignalWriter for UnWriteable<T> {
        type Target = T;

        &self -> &self.0;
        &mut self -> &mut self.0;
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

impl_deref_signal! {
    impl<T> _ + !SignalSeeker for UnSeekable<T> {
        type Target = T;

        &self -> &self.0;
        &mut self -> &mut self.0;
    }
}

impl<T: Signal> SignalSeeker for UnSeekable<T> {
    fn seek(&mut self, _offset: i64) -> PhonicResult<()> {
        Err(PhonicError::Unsupported)
    }
}

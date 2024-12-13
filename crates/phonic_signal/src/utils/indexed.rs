use crate::{
    delegate_signal, IndexedSignal, PhonicError, PhonicResult, Signal, SignalReader, SignalSeeker,
    SignalWriter,
};
use std::mem::MaybeUninit;

pub struct Indexed<T> {
    inner: T,
    pos: u64,
}

impl<T> Indexed<T> {
    pub fn new(inner: T) -> Self {
        Self { inner, pos: 0 }
    }
}

delegate_signal! {
    delegate<T> * + !IndexedSignal + !Mut for Indexed<T> {
        Self as T;

        &self => &self.inner;
    }
}

impl<T: Signal> IndexedSignal for Indexed<T> {
    fn pos(&self) -> u64 {
        self.pos
    }
}

impl<T: SignalReader> SignalReader for Indexed<T> {
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let n = self.inner.read(buf)?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl<T: SignalWriter> SignalWriter for Indexed<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let n = self.inner.write(buf)?;
        self.pos += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush()
    }
}

impl<T: SignalSeeker> SignalSeeker for Indexed<T> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        let pos = self
            .pos
            .checked_add_signed(offset)
            .ok_or(PhonicError::OutOfBounds)?;

        self.inner.seek(offset)?;
        self.pos = pos;

        Ok(())
    }
}

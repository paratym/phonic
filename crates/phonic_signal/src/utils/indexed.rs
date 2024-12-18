use crate::{
    delegate_signal, BlockingSignalReader, BlockingSignalWriter, BufferedSignal,
    BufferedSignalReader, BufferedSignalWriter, IndexedSignal, PhonicError, PhonicResult, Signal,
    SignalReader, SignalSeeker, SignalWriter,
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

impl<T: BufferedSignal> BufferedSignal for Indexed<T> {
    fn commit_samples(&mut self, n_samples: usize) {
        self.inner.commit_samples(n_samples);

        let n_frames = n_samples as u64 / self.spec().channels.count() as u64;
        self.pos += n_frames;
    }
}

impl<T: Signal> IndexedSignal for Indexed<T> {
    fn pos(&self) -> u64 {
        self.pos
    }
}

impl<T: SignalReader> SignalReader for Indexed<T> {
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let n_samples = self.inner.read(buf)?;
        let n_frames = n_samples as u64 / self.spec().channels.count() as u64;
        self.pos += n_frames;

        Ok(n_samples)
    }
}

impl<T: BufferedSignalReader> BufferedSignalReader for Indexed<T> {
    fn available_samples(&self) -> &[Self::Sample] {
        self.inner.available_samples()
    }
}

impl<T: BlockingSignalReader> BlockingSignalReader for Indexed<T> {
    fn read_blocking(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let n_samples = self.inner.read_blocking(buf)?;
        let n_frames = n_samples as u64 / self.spec().channels.count() as u64;
        self.pos += n_frames;

        Ok(n_samples)
    }
}

impl<T: SignalWriter> SignalWriter for Indexed<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let n_samples = self.inner.write(buf)?;
        let n_frames = n_samples as u64 / self.spec().channels.count() as u64;
        self.pos += n_frames;

        Ok(n_samples)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush()
    }
}

impl<T: BufferedSignalWriter> BufferedSignalWriter for Indexed<T> {
    fn available_slots(&mut self) -> &mut [MaybeUninit<Self::Sample>] {
        self.inner.available_slots()
    }
}

impl<T: BlockingSignalWriter> BlockingSignalWriter for Indexed<T> {
    fn write_blocking(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let n_samples = self.inner.write_blocking(buf)?;
        let n_frames = n_samples as u64 / self.spec().channels.count() as u64;
        self.pos += n_frames;

        Ok(n_samples)
    }

    fn flush_blocking(&mut self) -> PhonicResult<()> {
        self.inner.flush_blocking()
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

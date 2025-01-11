use crate::{
    delegate_signal, BufferedSignalReader, BufferedSignalWriter, IndexedSignal, IntoDuration,
    NFrames, NSamples, PhonicError, PhonicResult, Signal, SignalReader, SignalSeeker, SignalWriter,
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

    fn advance(&mut self, n_samples: usize)
    where
        Self: Signal,
    {
        let NFrames { n_frames } = NSamples::from(n_samples as u64).into_duration(self.spec());
        self.pos += n_frames;
    }
}

delegate_signal! {
    impl<T> * + !IndexedSignal + !Mut for Indexed<T> {
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
        let n_samples = self.inner.read(buf)?;
        self.advance(n_samples);

        Ok(n_samples)
    }
}

impl<T: BufferedSignalReader> BufferedSignalReader for Indexed<T> {
    fn fill(&mut self) -> PhonicResult<&[Self::Sample]> {
        self.inner.fill()
    }

    fn buffer(&self) -> Option<&[Self::Sample]> {
        self.inner.buffer()
    }

    fn consume(&mut self, n_samples: usize) {
        self.inner.consume(n_samples);
        self.advance(n_samples);
    }
}

impl<T: SignalWriter> SignalWriter for Indexed<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let n_samples = self.inner.write(buf)?;
        self.advance(n_samples);

        Ok(n_samples)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush()
    }
}

impl<T: BufferedSignalWriter> BufferedSignalWriter for Indexed<T> {
    fn buffer_mut(&mut self) -> Option<&mut [MaybeUninit<Self::Sample>]> {
        self.inner.buffer_mut()
    }

    fn commit(&mut self, n_samples: usize) {
        self.inner.commit(n_samples);
        self.advance(n_samples)
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

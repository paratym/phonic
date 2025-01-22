use phonic_signal::{
    delegate_signal,
    utils::{NFrames, SignalUtilsExt},
    FiniteSignal, IndexedSignal, PhonicError, PhonicResult, SignalReader, SignalSeeker,
};
use std::mem::MaybeUninit;

pub struct Repeat<T> {
    inner: T,
    reps: u32,
    current: u32,
}

impl<T> Repeat<T> {
    pub fn new(inner: T, reps: u32) -> Self {
        Self {
            inner,
            reps,
            current: 0,
        }
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

delegate_signal! {
    impl<T> Signal for Repeat<T> {
        Self as T;

        &self => &self.inner;
    }
}

impl<T: IndexedSignal + FiniteSignal> IndexedSignal for Repeat<T> {
    fn pos(&self) -> u64 {
        self.inner.len() * self.current as u64 + self.inner.pos()
    }
}

impl<T: FiniteSignal> FiniteSignal for Repeat<T> {
    fn len(&self) -> u64 {
        self.inner.len() * self.reps as u64
    }
}

impl<T: IndexedSignal + SignalReader + SignalSeeker> SignalReader for Repeat<T> {
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        while self.current < self.reps {
            let result = self.inner.read(buf);
            if result.as_ref().is_ok_and(|n| *n == 0) {
                self.inner.seek(0)?;
                self.current += 1;
                continue;
            }

            return result;
        }

        Ok(0)
    }
}

impl<T: IndexedSignal + FiniteSignal + SignalSeeker> SignalSeeker for Repeat<T> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        let pos = self
            .pos()
            .checked_add_signed(offset)
            .ok_or(PhonicError::OutOfBounds)?;

        if pos > self.len() {
            return Err(PhonicError::OutOfBounds);
        }

        let inner_pos = NFrames::from(pos % self.inner.len());
        self.inner.seek_from_start(inner_pos)

        // TODO: set current repetition
    }
}

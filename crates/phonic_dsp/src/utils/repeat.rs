use phonic_signal::{
    FiniteSignal, IndexedSignal, PhonicResult, SignalReader, SignalSeeker, SignalSpec,
};

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

impl<T: Signal> Signal for Repeat<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        self.inner.spec()
    }
}

impl<T: IndexedSignal + FiniteSignal> IndexedSignal for Repeat<T> {
    fn pos(&self) -> u64 {
        self.current as u64 + self.inner.len() + self.inner.pos()
    }
}

impl<T: FiniteSignal> FiniteSignal for Repeat<T> {
    fn len(&self) -> u64 {
        self.inner.len().saturating_mul(self.reps as u64)
    }
}

impl<T: IndexedSignal + SignalReader + SignalSeeker> SignalReader for Repeat<T> {
    fn read(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize> {
        while self.current < self.reps {
            let result = self.inner.read(buf);
            if !result.as_ref().is_ok_and(|n| *n == 0) {
                return result;
            }

            self.inner.seek_start()?;
            self.current += 1;
        }

        Ok(0)
    }
}

impl<T: IndexedSignal + FiniteSignal + SignalSeeker> SignalSeeker for Repeat<T> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        let pos = self.pos().checked_add_signed(offset);
        if pos.is_none_or(|pos| pos > self.len()) {
            return Err(PhonicError::OutOfBounds);
        }

        let inner_pos = pos.unwrap() % self.inner.len();
        self.inner.set_pos(inner_pos)
    }
}

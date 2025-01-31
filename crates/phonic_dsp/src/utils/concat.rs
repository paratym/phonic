use crate::types::SignalList;
use phonic_signal::{
    FiniteSignal, IndexedSignal, PhonicResult, Signal, SignalReader, SignalSeeker, SignalSpec,
    SignalWriter,
};
use std::mem::MaybeUninit;

pub struct Concat<T> {
    inner: T,
    spec: SignalSpec,
    idx: usize,
}

impl<T: SignalList> Concat<T> {
    pub fn new(inner: T) -> PhonicResult<Self> {
        let spec = inner.merged_spec()?;

        Ok(Self {
            inner,
            spec,
            idx: 0,
        })
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: SignalList> Signal for Concat<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: SignalList> IndexedSignal for Concat<T>
where
    for<'a> T::Signal<'a>: IndexedSignal,
{
    fn pos(&self) -> u64 {
        let range = 0..self.idx;
        range.map(|i| self.inner.signal(i).pos()).sum()
    }
}

impl<T: SignalList> FiniteSignal for Concat<T>
where
    for<'a> T::Signal<'a>: FiniteSignal,
{
    fn len(&self) -> u64 {
        let range = 0..self.inner.len();
        range.map(|i| self.inner.signal(i).len()).sum()
    }
}

impl<T: SignalList> SignalReader for Concat<T>
where
    for<'a> T::Signal<'a>: SignalReader,
{
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        while self.idx < self.inner.len() {
            match self.inner.signal(self.idx).read(buf) {
                Ok(0) => {
                    self.idx += 1;
                    continue;
                }

                result => return result,
            }
        }

        Ok(0)
    }
}

impl<T: SignalList> SignalWriter for Concat<T>
where
    for<'a> T::Signal<'a>: SignalWriter,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        while self.idx < self.inner.len() {
            match self.inner.signal(self.idx).write(buf) {
                Ok(0) => {
                    self.idx += 1;
                    continue;
                }

                result => return result,
            }
        }

        Ok(0)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        for i in 0..self.inner.len() {
            self.inner.signal(i).flush()?;
        }

        Ok(())
    }
}

impl<T: SignalList> SignalSeeker for Concat<T>
where
    for<'a> T::Signal<'a>: SignalSeeker,
{
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        todo!()
    }
}

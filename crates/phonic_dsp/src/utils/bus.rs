use crate::types::{FiniteSignalList, IndexedSignalList, PosQueue, SignalList, SignalWriterList};
use phonic_signal::{FiniteSignal, IndexedSignal, PhonicResult, Signal, SignalSpec, SignalWriter};

pub struct Bus<T> {
    inner: T,
    spec: SignalSpec,
    queue: PosQueue,
}

impl<T> Bus<T> {
    pub fn new(inner: T) -> PhonicResult<Self>
    where
        T: IndexedSignalList,
    {
        let spec = inner.spec()?;
        let queue = PosQueue::new(&inner);

        Ok(Self { inner, spec, queue })
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: SignalList> Signal for Bus<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: IndexedSignalList> IndexedSignal for Bus<T> {
    fn pos(&self) -> u64 {
        self.inner.min_pos()
    }
}

impl<T: FiniteSignalList> FiniteSignal for Bus<T> {
    fn len(&self) -> u64 {
        self.inner.max_len()
    }
}

impl<T: SignalWriterList> SignalWriter for Bus<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let Some(zero_cursor) = self.queue.peek_front().copied() else {
            return Ok(0);
        };

        let mut buf_len = buf.len();
        let n_channels = self.spec().channels.count() as usize;
        buf_len -= buf_len % n_channels;

        let mut n_written = 0;

        loop {
            let Some(cursor) = self.queue.peek_front() else {
                break;
            };

            let start_i = cursor.pos as usize - zero_cursor.pos as usize;
            if start_i >= buf_len {
                break;
            }

            let result = self.inner.write(cursor.id, &buf[start_i..buf_len]);

            match result {
                Ok(0) => {
                    self.queue.pop_front();
                }
                Ok(n) => {
                    n_written = match n_written {
                        0 => n,
                        _ => n_written.min(start_i + n),
                    };

                    debug_assert_eq!(n % n_channels, 0);
                    self.queue.commit_front(n as u64 / n_channels as u64);
                }
                Err(e) => {
                    n_written = n_written.min(start_i);

                    if n_written == 0 {
                        return Err(e);
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(n_written)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        let mut range = 0..self.inner.count();
        range.try_for_each(|i| self.inner.flush(i))
    }
}

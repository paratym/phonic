use crate::types::{PosQueue, SignalList};
use phonic_signal::{FiniteSignal, IndexedSignal, PhonicResult, Signal, SignalSpec, SignalWriter};

pub struct Bus<T> {
    inner: T,
    spec: SignalSpec,
    queue: PosQueue,
}

impl<T> Bus<T> {
    pub fn new(inner: T) -> PhonicResult<Self>
    where
        T: SignalList,
        for<'a> T::Signal<'a>: IndexedSignal,
    {
        let spec = inner.merged_spec()?;
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

impl<T: SignalList> IndexedSignal for Bus<T>
where
    for<'a> T::Signal<'a>: IndexedSignal,
{
    fn pos(&self) -> u64 {
        let range = 0..self.inner.len();

        range
            .map(|i| self.inner.signal(i).pos())
            .min()
            .unwrap_or_default()
    }
}

impl<T: SignalList> FiniteSignal for Bus<T>
where
    for<'a> T::Signal<'a>: FiniteSignal,
{
    fn len(&self) -> u64 {
        let range = 0..self.inner.len();

        range
            .map(|i| self.inner.signal(i).len())
            .max()
            .unwrap_or_default()
    }
}

impl<T: SignalList> SignalWriter for Bus<T>
where
    for<'a> T::Signal<'a>: SignalWriter,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let Some(zero_cursor) = self.queue.peek_front().copied() else {
            return Ok(0);
        };

        let mut buf_len = buf.len();
        let n_channels = self.spec().n_channels;
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

            let result = self.inner.signal(cursor.id).write(&buf[start_i..buf_len]);

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
        for i in 0..self.inner.len() {
            self.inner.signal(i).flush()?;
        }

        Ok(())
    }
}

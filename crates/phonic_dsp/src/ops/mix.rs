use crate::{
    ops::Complement,
    types::{FiniteSignalList, IndexedSignalList, PosQueue, SignalList, SignalReaderList},
};
use phonic_signal::{
    utils::DefaultBuf, FiniteSignal, IndexedSignal, PhonicResult, Sample, Signal, SignalReader,
    SignalSpec,
};
use std::ops::DerefMut;

pub struct Mix<T: SignalList, B = DefaultBuf<<T as SignalList>::Sample>> {
    inner: T,
    spec: SignalSpec,
    queue: PosQueue,
    buf: B,
    partial_start: usize,
    partial_end: usize,
}

pub trait MixSample {
    fn mix(self, other: Self) -> Self;
}

impl<T: IndexedSignalList, B> Mix<T, B> {
    pub fn new(inner: T) -> PhonicResult<Self>
    where
        B: Default,
    {
        Self::new_buffered(inner, B::default())
    }

    pub fn new_buffered(inner: T, buf: B) -> PhonicResult<Self> {
        let spec = inner.spec()?;
        let queue = PosQueue::new(&inner);

        Ok(Self {
            inner,
            spec,
            queue,
            buf,
            partial_start: 0,
            partial_end: 0,
        })
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T, C, B> Mix<(T, Complement<C>), B>
where
    (T, Complement<C>): IndexedSignalList,
{
    pub fn cancel(inner: T, other: C) -> PhonicResult<Self>
    where
        B: Default,
    {
        let complement = Complement::new(other);
        Self::new((inner, complement))
    }

    pub fn cancel_buffered(inner: T, other: C, buf: B) -> PhonicResult<Self> {
        let complement = Complement::new(other);
        Self::new_buffered((inner, complement), buf)
    }
}

impl<T, B> Mix<T, B>
where
    T: SignalList,
    B: DerefMut<Target = [T::Sample]>,
{
    fn take_partial_samples(&mut self, buf: &mut [T::Sample]) -> usize {
        let partial_len = self.partial_end - self.partial_start;
        let buf_len = buf.len().min(partial_len);
        let partial_buf = &self.buf[self.partial_start..self.partial_start + buf_len];
        buf[..buf_len].copy_from_slice(partial_buf);

        if buf_len == partial_len {
            self.partial_start = 0;
            self.partial_end = 0;
        } else {
            self.partial_start += buf_len;
        }

        buf_len
    }

    fn put_partial_samples(&mut self, buf: &[T::Sample]) {
        let buf_len = buf.len();
        let start_i = if self.partial_start < self.partial_end {
            self.partial_start as isize - buf_len as isize
        } else {
            0
        };

        if start_i.is_negative() {
            todo!()
        }

        let start_i = start_i as usize;
        self.buf[start_i..start_i + buf_len].copy_from_slice(buf);
    }
}

impl<T: SignalList, B> Signal for Mix<T, B> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: IndexedSignalList, B> IndexedSignal for Mix<T, B> {
    fn pos(&self) -> u64 {
        self.inner.min_pos()
    }
}

impl<T: FiniteSignalList, B> FiniteSignal for Mix<T, B> {
    fn len(&self) -> u64 {
        self.inner.max_len()
    }
}

impl<T, B> SignalReader for Mix<T, B>
where
    T: IndexedSignalList + SignalReaderList,
    T::Sample: MixSample,
    B: DerefMut<Target = [T::Sample]>,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize> {
        let Some(zero_cursor) = self.queue.peek_front().copied() else {
            return Ok(0);
        };

        let mut buf_len = buf.len().min(self.buf.len());
        let n_channels = self.spec().channels.count() as usize;
        buf_len -= buf_len % n_channels;

        let partial_len = self.take_partial_samples(&mut buf[..buf_len]);
        buf[partial_len..buf_len].fill(T::Sample::ORIGIN);

        let mut max_read = partial_len;
        let mut min_read = 0;

        loop {
            let Some(cursor) = self.queue.peek_front().copied() else {
                break;
            };

            let start_frame = cursor.pos - zero_cursor.pos;
            let start_i = start_frame as usize * n_channels;
            if start_i >= buf_len {
                break;
            }

            let inner_buf = &mut self.buf[start_i..buf_len];
            let result = self.inner.read(cursor.id, inner_buf);

            match result {
                Ok(0) => {
                    self.queue.pop_front();
                }
                Ok(n) => {
                    let end_i = start_i + n;
                    max_read = max_read.max(end_i);
                    min_read = match min_read {
                        0 => n,
                        _ => min_read.min(end_i),
                    };

                    inner_buf[..n]
                        .iter()
                        .zip(&mut buf[start_i..end_i])
                        .for_each(|(s, mix)| *mix = mix.mix(*s));

                    debug_assert_eq!(n % n_channels, 0);
                    self.queue.commit_front(n as u64 / n_channels as u64);
                }
                Err(e) => {
                    min_read = min_read.min(start_i);

                    if min_read == 0 {
                        return Err(e);
                    } else {
                        break;
                    }
                }
            }
        }

        self.put_partial_samples(&buf[min_read..max_read]);
        Ok(min_read)
    }
}

macro_rules! impl_mix {
    ($sample:ident, $name:ident, $other:ident, $func:expr) => {
        impl MixSample for $sample {
            #[inline]
            fn mix(self, $other: Self) -> Self {
                let $name = self;
                $func
            }
        }
    };
}

macro_rules! impl_unsigned_mix {
    ($sample:ident, $name:ident, $other:ident) => {
        impl_mix!($sample, $name, $other, {
            let amp = $other.abs_diff($sample::ORIGIN);
            if $other >= $sample::ORIGIN {
                $name.saturating_add(amp)
            } else {
                $name.saturating_sub(amp)
            }
        });
    };
}

impl_mix!(i8, a, b, a.saturating_add(b));
impl_mix!(i16, a, b, a.saturating_add(b));
impl_mix!(i32, a, b, a.saturating_add(b));
impl_mix!(i64, a, b, a.saturating_add(b));

impl_unsigned_mix!(u8, a, b);
impl_unsigned_mix!(u16, a, b);
impl_unsigned_mix!(u32, a, b);
impl_unsigned_mix!(u64, a, b);

impl_mix!(f32, a, b, a + b);
impl_mix!(f64, a, b, a + b);

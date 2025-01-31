use crate::{
    ops::Complement,
    types::{PosQueue, SignalList},
};
use phonic_signal::{
    utils::{slice_as_init_mut, DefaultSizedBuf},
    FiniteSignal, IndexedSignal, PhonicResult, Sample, Signal, SignalExt, SignalReader, SignalSpec,
};
use std::{mem::MaybeUninit, ops::Add};

pub struct Mix<T: SignalList, B = DefaultSizedBuf<MaybeUninit<<T as SignalList>::Sample>>> {
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

impl<T: SignalList, B> Mix<T, B> {
    pub fn new(inner: T, buf: B) -> PhonicResult<Self>
    where
        for<'a> T::Signal<'a>: IndexedSignal,
    {
        let spec = inner.merged_spec()?;
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
    (T, Complement<C>): SignalList,
    for<'a> <(T, Complement<C>) as SignalList>::Signal<'a>: IndexedSignal,
{
    pub fn cancel(inner: T, other: C, buf: B) -> PhonicResult<Self> {
        let complement = Complement::new(other);
        Self::new((inner, complement), buf)
    }
}

impl<T, B> Mix<T, B>
where
    T: SignalList,
    B: AsMut<[MaybeUninit<T::Sample>]>,
{
    fn take_partial_samples(&mut self, buf: &mut [MaybeUninit<T::Sample>]) -> usize {
        let partial_len = self.partial_end - self.partial_start;
        let buf_len = buf.len().min(partial_len);
        let partial_buf = &self.buf.as_mut()[self.partial_start..self.partial_start + buf_len];
        buf[..buf_len].copy_from_slice(partial_buf);

        if buf_len == partial_len {
            self.partial_start = 0;
            self.partial_end = 0;
        } else {
            self.partial_start += buf_len;
        }

        buf_len
    }

    fn put_partial_samples(&mut self, buf: &[MaybeUninit<T::Sample>]) {
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
        self.buf.as_mut()[start_i..start_i + buf_len].copy_from_slice(buf);
    }
}

impl<T: SignalList, B> Signal for Mix<T, B> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: SignalList, B> IndexedSignal for Mix<T, B>
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

impl<T: SignalList, B> FiniteSignal for Mix<T, B>
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

impl<T, B> SignalReader for Mix<T, B>
where
    T: SignalList,
    T::Sample: MixSample,
    for<'a> T::Signal<'a>: SignalReader,
    B: AsMut<[MaybeUninit<Self::Sample>]>,
{
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let Some(zero_cursor) = self.queue.peek_front().copied() else {
            return Ok(0);
        };

        let mut buf_len = buf.len().min(self.buf.as_mut().len());
        let n_channels = self.spec().n_channels;
        buf_len -= buf_len % n_channels;

        let partial_len = self.take_partial_samples(&mut buf[..buf_len]);
        buf[partial_len..buf_len].fill(MaybeUninit::new(T::Sample::ORIGIN));
        let init_buf = unsafe { slice_as_init_mut(&mut buf[..buf_len]) };

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

            let inner_buf = &mut self.buf.as_mut()[start_i..buf_len];
            let result = self.inner.signal(cursor.id).read_init(inner_buf);

            match result {
                Ok([]) => {
                    self.queue.pop_front();
                }
                Ok(samples) => {
                    let n_samples = samples.len();
                    let end_i = start_i + n_samples;
                    max_read = max_read.max(end_i);
                    min_read = match min_read {
                        0 => n_samples,
                        _ => min_read.min(end_i),
                    };

                    samples
                        .iter()
                        .zip(&mut init_buf[start_i..end_i])
                        .for_each(|(s, mix)| *mix = mix.mix(*s));

                    debug_assert_eq!(n_samples % n_channels, 0);
                    self.queue
                        .commit_front(n_samples as u64 / n_channels as u64);
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
    ($sample:ident, $self:ident, $other:ident, $func:expr) => {
        impl MixSample for $sample {
            #[inline]
            fn mix($self, $other: Self) -> Self {
                $func
            }
        }
    };
}

macro_rules! impl_unsigned_mix {
    ($sample:ident, $self:ident, $other:ident) => {
        impl_mix!($sample, $self, $other, {
            if $other >= $sample::ORIGIN {
                $self.saturating_add($other - $sample::ORIGIN)
            } else {
                $self.saturating_sub($sample::ORIGIN - $other)
            }
        });
    };
}

impl_mix!(i8, self, s, self.saturating_add(s));
impl_mix!(i16, self, s, self.saturating_add(s));
impl_mix!(i32, self, s, self.saturating_add(s));
impl_mix!(i64, self, s, self.saturating_add(s));

impl_unsigned_mix!(u8, self, s);
impl_unsigned_mix!(u16, self, s);
impl_unsigned_mix!(u32, self, s);
impl_unsigned_mix!(u64, self, s);

impl_mix!(f32, self, s, self.add(s));
impl_mix!(f64, self, s, self.add(s));

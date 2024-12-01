use phonic_signal::{
    utils::DefaultBuf, FiniteSignal, IndexedSignal, PhonicResult, Sample, Signal, SignalReader,
    SignalSeeker, SignalSpec, SignalWriter,
};
use std::{marker::PhantomData, ops::DerefMut};

pub struct Convert<T: Signal, S: Sample, B = DefaultBuf<<T as Signal>::Sample>> {
    inner: T,
    buf: B,
    _sample: PhantomData<S>,
}

pub trait FromSample<S: Sample> {
    fn from_sample(sample: S) -> Self;
}

pub trait IntoSample<S: Sample> {
    fn into_sample(self) -> S;
}

impl<T, S, B> Convert<T, S, B>
where
    T: Signal,
    S: Sample,
{
    pub fn new_buffered(inner: T, buf: B) -> Self {
        Self {
            inner,
            buf,
            _sample: PhantomData,
        }
    }

    pub fn new(inner: T) -> Self
    where
        B: Default,
    {
        Self {
            inner,
            buf: B::default(),
            _sample: PhantomData,
        }
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T, S, B> Signal for Convert<T, S, B>
where
    T: Signal,
    S: Sample,
{
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        self.inner.spec()
    }
}

impl<T, S, B> IndexedSignal for Convert<T, S, B>
where
    T: IndexedSignal,
    S: Sample,
{
    fn pos(&self) -> u64 {
        self.inner.pos()
    }
}

impl<T, S, B> FiniteSignal for Convert<T, S, B>
where
    T: FiniteSignal,
    S: Sample,
{
    fn len(&self) -> u64 {
        self.inner.len()
    }
}

impl<T, S, B> SignalReader for Convert<T, S, B>
where
    T: SignalReader,
    T::Sample: IntoSample<S>,
    S: Sample,
    B: DerefMut<Target = [T::Sample]>,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize> {
        let buf_len = buf.len().min(self.buf.len());
        let n = self.inner.read(&mut self.buf[..buf_len])?;

        self.buf
            .iter()
            .zip(buf[..n].iter_mut())
            .for_each(|(inner, outer)| *outer = inner.into_sample());

        Ok(n)
    }
}

impl<T, S, B> SignalWriter for Convert<T, S, B>
where
    T: SignalWriter,
    S: Sample + IntoSample<T::Sample>,
    B: DerefMut<Target = [T::Sample]>,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let buf_len = buf.len().min(self.buf.len());

        self.buf
            .iter_mut()
            .zip(buf[..buf_len].iter())
            .for_each(|(inner, outer)| *inner = outer.into_sample());

        self.inner.write(&self.buf[..buf_len])
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush()
    }
}

impl<T, S, B> SignalSeeker for Convert<T, S, B>
where
    T: SignalSeeker,
    S: Sample,
{
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        self.inner.seek(offset)
    }
}

impl<T: Sample, S: Sample + FromSample<T>> IntoSample<S> for T {
    #[inline(always)]
    fn into_sample(self) -> S {
        S::from_sample(self)
    }
}

macro_rules! impl_convert {
    ($sample:ty as $name:ident, $($into:ty => $func:expr),+) => {
        $(impl FromSample<$sample> for $into {
            #[inline]
            fn from_sample($name: $sample) -> Self {
                $func
            }
        })+
    };
}

impl_convert!(
    i8 as s,

    // signed
    i8 => s,
    i16 => (s as i16) << 8,
    i32 => (s as i32) << 24,
    i64 => (s as i64) << 56,

    // unsigned
    u8 => (s as u8).wrapping_add(1 << 7),
    u16 => IntoSample::<u8>::into_sample(s).into_sample(),
    u32 => IntoSample::<u8>::into_sample(s).into_sample(),
    u64 => IntoSample::<u8>::into_sample(s).into_sample(),

    // float
    f32 => s as f32 / (i8::MAX as f32 + 1.0),
    f64 => s as f64 / (i8::MAX as f64 + 1.0)
);

impl_convert!(
    i16 as s,

    // signed
    i8 => (s >> 8) as i8,
    i16 => s,
    i32 => (s as i32) << 16,
    i64 => (s as i64) << 48,

    // unsigned
    u8 => IntoSample::<u16>::into_sample(s).into_sample(),
    u16 => (s as u16).wrapping_add(1 << 15),
    u32 => IntoSample::<u16>::into_sample(s).into_sample(),
    u64 => IntoSample::<u16>::into_sample(s).into_sample(),

    // float
    f32 => s as f32 / (i16::MAX as f32 + 1.0),
    f64 => s as f64 / (i16::MAX as f64 + 1.0)
);

impl_convert!(
    i32 as s,

    // signed
    i8 => (s >> 24) as i8,
    i16 => (s >> 16) as i16,
    i32 => s,
    i64 => (s as i64) << 32,

    // unsigned
    u8 => IntoSample::<u32>::into_sample(s).into_sample(),
    u16 => IntoSample::<u32>::into_sample(s).into_sample(),
    u32 => (s as u32).wrapping_add(1 << 31),
    u64 => IntoSample::<u32>::into_sample(s).into_sample(),

    // float
    f32 => IntoSample::<f64>::into_sample(s).into_sample(),
    f64 => s as f64 / (i32::MAX as f64 + 1.0)
);

impl_convert!(
    i64 as s,

    // signed
    i8 => (s >> 56) as i8,
    i16 => (s >> 48) as i16,
    i32 => (s >> 32) as i32,
    i64 => s,

    // unsigned
    u8 => IntoSample::<u64>::into_sample(s).into_sample(),
    u16 => IntoSample::<u64>::into_sample(s).into_sample(),
    u32 => IntoSample::<u64>::into_sample(s).into_sample(),
    u64 => (s as u64).wrapping_add(1 << 63),

    // float
    f32 => IntoSample::<f64>::into_sample(s).into_sample(),
    f64 => s as f64 / (i64::MAX as f64 + 1.0)
);

impl_convert!(
    u8 as s,

    // signed
    i8 => s.wrapping_sub(1 << 7) as i8,
    i16 => IntoSample::<i8>::into_sample(s).into_sample(),
    i32 => IntoSample::<i8>::into_sample(s).into_sample(),
    i64 => IntoSample::<i8>::into_sample(s).into_sample(),

    // unsigned
    u8 => s,
    u16 => (s as u16) << 8,
    u32 => (s as u32) << 24,
    u64 => (s as u64) << 56,

    // float
    f32 => ((s as f32) / (u8::ORIGIN as f32 + 1.0)) - 1.0,
    f64 => ((s as f64) / (u8::ORIGIN as f64 + 1.0)) - 1.0
);

impl_convert!(
    u16 as s,

    // signed
    i8 => IntoSample::<i16>::into_sample(s).into_sample(),
    i16 => s.wrapping_sub(1 << 15) as i16,
    i32 => IntoSample::<i16>::into_sample(s).into_sample(),
    i64 => IntoSample::<i16>::into_sample(s).into_sample(),

    // unsigned
    u8 => (s >> 8) as u8,
    u16 => s,
    u32 => (s as u32) << 16,
    u64 => (s as u64) << 48,

    // float
    f32 => ((s as f32) / (u16::ORIGIN as f32 + 1.0)) - 1.0,
    f64 => ((s as f64) / (u16::ORIGIN as f64 + 1.0)) - 1.0
);

impl_convert!(
    u32 as s,

    // signed
    i8 => IntoSample::<i32>::into_sample(s).into_sample(),
    i16 => IntoSample::<i32>::into_sample(s).into_sample(),
    i32 => s.wrapping_sub(1 << 31) as i32,
    i64 => IntoSample::<i32>::into_sample(s).into_sample(),

    // unsigned
    u8 => (s >> 24) as u8,
    u16 => (s >> 16) as u16,
    u32 => s,
    u64 => (s as u64) << 32,

    // float
    f32 => IntoSample::<f64>::into_sample(s).into_sample(),
    f64 => ((s as f64) / (u32::ORIGIN as f64 + 1.0)) - 1.0
);

impl_convert!(
    u64 as s,

    // signed
    i8 => IntoSample::<i64>::into_sample(s).into_sample(),
    i16 => IntoSample::<i64>::into_sample(s).into_sample(),
    i32 => IntoSample::<i64>::into_sample(s).into_sample(),
    i64 => s.wrapping_sub(1 << 63) as i64,

    // unsigned
    u8 => (s >> 56) as u8,
    u16 => (s >> 48) as u16,
    u32 => (s >> 32) as u32,
    u64 => s,

    // float
    f32 => IntoSample::<f64>::into_sample(s).into_sample(),
    f64 => ((s as f64) / (u64::ORIGIN as f64 + 1.0)) - 1.0
);

impl_convert!(
    f32 as s,

    // signed
    i8 => (s * i8::MAX as f32) as i8,
    i16 => (s * i16::MAX as f32) as i16,
    i32 => IntoSample::<f64>::into_sample(s).into_sample(),
    i64 => IntoSample::<f64>::into_sample(s).into_sample(),

    // unsigned
    u8 => IntoSample::<i8>::into_sample(s).into_sample(),
    u16 => IntoSample::<i16>::into_sample(s).into_sample(),
    u32 => IntoSample::<i32>::into_sample(s).into_sample(),
    u64 => IntoSample::<i64>::into_sample(s).into_sample(),

    // float
    f32 => s,
    f64 => s as f64
);

impl_convert!(
    f64 as s,

    // signed
    i8 => (s * i8::MAX as f64) as i8,
    i16 => (s * i16::MAX as f64) as i16,
    i32 => (s * i32::MAX as f64) as i32,
    i64 => (s * i64::MAX as f64) as i64,

    // unsigned
    u8 => IntoSample::<i8>::into_sample(s).into_sample(),
    u16 => IntoSample::<i16>::into_sample(s).into_sample(),
    u32 => IntoSample::<i32>::into_sample(s).into_sample(),
    u64 => IntoSample::<i64>::into_sample(s).into_sample(),

    // float
    f32 => s as f32,
    f64 => s
);

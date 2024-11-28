use phonic_signal::{
    DefaultBuf, FiniteSignal, IndexedSignal, PhonicResult, Sample, Signal, SignalReader,
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

#[inline(always)]
fn i8_to_u8(s: i8) -> u8 {
    (s as u8).wrapping_add(0x80)
}

impl_convert!(
    i8 as s,
    // signed
    i8 => s,
    i16 => (s as i16) << 8,
    i32 => (s as i32) << 24,
    i64 => (s as i64) << 56,
    // unsigned
    u8 => i8_to_u8(s),
    u16 => (i8_to_u8(s) as u16) << 8,
    u32 => (i8_to_u8(s) as u32) << 24,
    u64 => todo!(),
    // float
    f32 => s as f32 / 128.0,
    f64 => s as f64 / 128.0
);

#[inline(always)]
fn i16_to_u16(s: i16) -> u16 {
    (s as u16).wrapping_add(0x8000)
}

impl_convert!(
    i16 as s,
    // signed
    i8 => (s >> 8) as i8,
    i16 => s,
    i32 => (s as i32) << 16,
    i64 => (s as i64) << 56,
    // unsigned
    u8 => (i16_to_u16(s) >> 8) as u8,
    u16 => i16_to_u16(s),
    u32 => (i16_to_u16(s) as u32) << 16,
    u64 => todo!(),
    // float
    f32 => s as f32 / 32_768.0,
    f64 => s as f64 / 32_768.0
);

#[inline(always)]
fn i32_to_u32(s: i32) -> u32 {
    (s as u32).wrapping_add(0x8000_0000)
}

impl_convert!(
    i32 as s,
    // signed
    i8 => (s >> 24) as i8,
    i16 => (s >> 16) as i16,
    i32 => s,
    i64 => todo!(),
    // unsigned
    u8 => (i32_to_u32(s) >> 24) as u8,
    u16 => (i32_to_u32(s) >> 16) as u16,
    u32 => i32_to_u32(s),
    u64 => todo!(),
    // float
    f32 => (s as f64 / 2_147_483_648.0) as f32,
    f64 => s as f64 / 2_147_483_648.0
);

impl_convert!(
    i64 as s,
    // signed
    i8 => todo!(),
    i16 => todo!(),
    i32 => todo!(),
    i64 => s,
    // unsigned
    u8 => todo!(),
    u16 => todo!(),
    u32 => todo!(),
    u64 => todo!(),
    // float
    f32 => todo!(),
    f64 => todo!()
);

impl_convert!(
    u8 as s,
    // signed
    i8 => s.wrapping_sub(0x80) as i8,
    i16 => ((s.wrapping_sub(0x80) as i8) as i16) << 8,
    i32 => ((s.wrapping_sub(0x80) as i8) as i32) << 24,
    i64 => ((s.wrapping_sub(0x80) as i8) as i64) << 56,
    // unsigned
    u8 => s,
    u16 => (s as u16) << 8,
    u32 => (s as u32) << 24,
    u64 => (s as u64) << 56,
    // float
    f32 => ((s as f32) / 128.0) - 1.0,
    f64 => ((s as f64) / 128.0) - 1.0
);

impl_convert!(
    u16 as s,
    // signed
    i8 => (s.wrapping_sub(0x8000) >> 8) as i8,
    i16 => s.wrapping_sub(0x8000) as i16,
    i32 => ((s.wrapping_sub(0x8000) as i16) as i32) << 16,
    i64 => ((s.wrapping_sub(0x8000) as i16) as i64) << 48,
    // unsigned
    u8 => (s >> 8) as u8,
    u16 => s,
    u32 => (s as u32) << 16,
    u64 => (s as u64) << 48,
    // float
    f32 => ((s as f32) / 32_768.0) - 1.0,
    f64 => ((s as f64) / 32_768.0) - 1.0
);

impl_convert!(
    u32 as s,
    // signed
    i8 => (s.wrapping_sub(0x8000_0000) >> 24) as i8,
    i16 => (s.wrapping_sub(0x8000_0000) >> 16) as i16,
    i32 => s.wrapping_sub(0x8000_0000) as i32,
    i64 => ((s.wrapping_sub(0x8000_0000) as i32) as i64) << 48,
    // unsigned
    u8 => (s >> 24) as u8,
    u16 => (s >> 16) as u16,
    u32 => s,
    u64 => (s as u64) << 32,
    // float
    f32 => (((s as f64) / 2_147_483_648.0) - 1.0) as f32,
    f64 => ((s as f64) / 2_147_483_648.0) - 1.0
);

impl_convert!(
    u64 as s,
    // signed
    i8 => todo!(),
    i16 => todo!(),
    i32 => todo!(),
    i64 => todo!(),
    // unsigned
    u8 => todo!(),
    u16 => todo!(),
    u32 => todo!(),
    u64 => s,
    // float
    f32 => todo!(),
    f64 => todo!()
);

impl_convert!(
    f32 as s,
    // signed
    i8 => todo!(),
    i16 => todo!(),
    i32 => todo!(),
    i64 => todo!(),
    // unsigned
    u8 => todo!(),
    u16 => todo!(),
    u32 => todo!(),
    u64 => todo!(),
    // float
    f32 => s,
    f64 => s as f64
);

impl_convert!(
    f64 as s,
    // signed
    i8 => todo!(),
    i16 => todo!(),
    i32 => todo!(),
    i64 => todo!(),
    // unsigned
    u8 => todo!(),
    u16 => todo!(),
    u32 => todo!(),
    u64 => todo!(),
    // float
    f32 => s as f32,
    f64 => s
);

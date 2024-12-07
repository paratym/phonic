use crate::ops::ClipSample;
use phonic_macro::impl_deref_signal;
use phonic_signal::{
    utils::DefaultBuf, PhonicResult, Sample, Signal, SignalReader, SignalSeeker, SignalSpec,
    SignalWriter,
};
use std::{
    marker::PhantomData,
    mem::{transmute, MaybeUninit},
    ops::DerefMut,
};

pub struct Convert<T: Signal, S: Sample, B = DefaultBuf<MaybeUninit<<T as Signal>::Sample>>> {
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

impl_deref_signal! {
    impl<T, S: Sample, B> _ + !Signal for Convert<T, S, B> {
        type Target = T;

        &self -> &self.inner;
   }
}

impl<T: Signal, S: Sample, B> Signal for Convert<T, S, B> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        self.inner.spec()
    }
}

impl<T, S, B> SignalReader for Convert<T, S, B>
where
    T: SignalReader,
    T::Sample: IntoSample<S>,
    S: Sample,
    B: DerefMut<Target = [MaybeUninit<T::Sample>]>,
{
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let buf_len = buf.len().min(self.buf.len());
        let samples = self.inner.read_init(&mut self.buf[..buf_len])?;

        buf.iter_mut()
            .zip(samples.iter())
            .for_each(|(outer, inner)| {
                outer.write(inner.into_sample());
            });

        Ok(samples.len())
    }
}

impl<T, S, B> SignalWriter for Convert<T, S, B>
where
    T: SignalWriter,
    S: Sample + IntoSample<T::Sample>,
    B: DerefMut<Target = [MaybeUninit<T::Sample>]>,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let iter = buf.iter().zip(self.buf.iter_mut());
        let len = iter.len();

        buf.iter()
            .zip(self.buf.iter_mut())
            .for_each(|(outer, inner)| {
                inner.write(outer.into_sample());
            });

        let uninit_slice = &self.buf[..len];
        let init_slice =
            unsafe { transmute::<&[MaybeUninit<T::Sample>], &[T::Sample]>(uninit_slice) };

        self.inner.write(init_slice)
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
            #[inline(always)]
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
    u8 => (s as u8).wrapping_add(const { 1 << 7 }),
    u16 => u8::from_sample(s).into_sample(),
    u32 => u8::from_sample(s).into_sample(),
    u64 => u8::from_sample(s).into_sample(),

    // float
    f32 => s as f32 / const { i8::MAX as f32 + 1.0 },
    f64 => s as f64 / const { i8::MAX as f64 + 1.0 }
);

impl_convert!(
    i16 as s,

    // signed
    i8 => (s >> 8) as i8,
    i16 => s,
    i32 => (s as i32) << 16,
    i64 => (s as i64) << 48,

    // unsigned
    u8 => u16::from_sample(s).into_sample(),
    u16 => (s as u16).wrapping_add(const { 1 << 15 }),
    u32 => u16::from_sample(s).into_sample(),
    u64 => u16::from_sample(s).into_sample(),

    // float
    f32 => s as f32 / const { i16::MAX as f32 + 1.0 },
    f64 => s as f64 / const { i16::MAX as f64 + 1.0 }
);

impl_convert!(
    i32 as s,

    // signed
    i8 => (s >> 24) as i8,
    i16 => (s >> 16) as i16,
    i32 => s,
    i64 => (s as i64) << 32,

    // unsigned
    u8 => u32::from_sample(s).into_sample(),
    u16 => u32::from_sample(s).into_sample(),
    u32 => (s as u32).wrapping_add(const { 1 << 31 }),
    u64 => u32::from_sample(s).into_sample(),

    // float
    f32 => IntoSample::<f64>::into_sample(s).into_sample(),
    f64 => s as f64 / const { i32::MAX as f64 + 1.0 }
);

impl_convert!(
    i64 as s,

    // signed
    i8 => (s >> 56) as i8,
    i16 => (s >> 48) as i16,
    i32 => (s >> 32) as i32,
    i64 => s,

    // unsigned
    u8 => u64::from_sample(s).into_sample(),
    u16 => u64::from_sample(s).into_sample(),
    u32 => u64::from_sample(s).into_sample(),
    u64 => (s as u64).wrapping_add(const { 1 << 63 }),

    // float
    f32 => f64::from_sample(s).into_sample(),
    f64 => s as f64 / const { i64::MAX as f64 + 1.0 }
);

impl_convert!(
    u8 as s,

    // signed
    i8 => s.wrapping_sub(const { 1 << 7 }) as i8,
    i16 => i8::from_sample(s).into_sample(),
    i32 => i8::from_sample(s).into_sample(),
    i64 => i8::from_sample(s).into_sample(),

    // unsigned
    u8 => s,
    u16 => (s as u16) << 8,
    u32 => (s as u32) << 24,
    u64 => (s as u64) << 56,

    // float
    f32 => ((s as f32) / const { u8::ORIGIN as f32 }) - 1.0,
    f64 => ((s as f64) / const { u8::ORIGIN as f64 }) - 1.0
);

impl_convert!(
    u16 as s,

    // signed
    i8 => i16::from_sample(s).into_sample(),
    i16 => s.wrapping_sub(const { 1 << 15 }) as i16,
    i32 => i16::from_sample(s).into_sample(),
    i64 => i16::from_sample(s).into_sample(),

    // unsigned
    u8 => (s >> 8) as u8,
    u16 => s,
    u32 => (s as u32) << 16,
    u64 => (s as u64) << 48,

    // float
    f32 => ((s as f32) / const { u16::ORIGIN as f32 }) - 1.0,
    f64 => ((s as f64) / const { u16::ORIGIN as f64 }) - 1.0
);

impl_convert!(
    u32 as s,

    // signed
    i8 => i32::from_sample(s).into_sample(),
    i16 => i32::from_sample(s).into_sample(),
    i32 => s.wrapping_sub(const { 1 << 31 }) as i32,
    i64 => i32::from_sample(s).into_sample(),

    // unsigned
    u8 => (s >> 24) as u8,
    u16 => (s >> 16) as u16,
    u32 => s,
    u64 => (s as u64) << 32,

    // float
    f32 => f64::from_sample(s).into_sample(),
    f64 => ((s as f64) / const { u32::ORIGIN as f64 }) - 1.0
);

impl_convert!(
    u64 as s,

    // signed
    i8 => i64::from_sample(s).into_sample(),
    i16 => i64::from_sample(s).into_sample(),
    i32 => i64::from_sample(s).into_sample(),
    i64 => s.wrapping_sub(const { 1 << 63 }) as i64,

    // unsigned
    u8 => (s >> 56) as u8,
    u16 => (s >> 48) as u16,
    u32 => (s >> 32) as u32,
    u64 => s,

    // float
    f32 => f64::from_sample(s).into_sample(),
    f64 => ((s as f64) / const { u64::ORIGIN as f64 }) - 1.0
);

impl_convert!(
    f32 as s,

    // signed
    i8 => (s.clip() * const { i8::MAX as f32 + 1.0 }) as i8,
    i16 => (s.clip() * const { i16::MAX as f32 + 1.0 }) as i16,
    i32 => f64::from_sample(s).into_sample(),
    i64 => f64::from_sample(s).into_sample(),

    // unsigned
    u8 => ((s.clip() + 1.0) * const { u8::ORIGIN as f32 }) as u8,
    u16 => ((s.clip() + 1.0) * const { u16::ORIGIN  as f32 }) as u16,
    u32 => f64::from_sample(s).into_sample(),
    u64 => f64::from_sample(s).into_sample(),

    // float
    f32 => s,
    f64 => s as f64
);

impl_convert!(
    f64 as s,

    // signed
    i8 => (s.clip() * const { i8::MAX as f64 + 1.0 }) as i8,
    i16 => (s.clip() * const { i16::MAX as f64 + 1.0 }) as i16,
    i32 => (s.clip() * const { i32::MAX as f64 + 1.0 }) as i32,
    i64 => (s.clip() * const { i64::MAX as f64 + 1.0 }) as i64,

    // unsigned
    u8 => ((s.clip() + 1.0) * const { u8::ORIGIN as f64 }) as u8,
    u16 => ((s.clip() + 1.0) * const { u16::ORIGIN as f64 }) as u16,
    u32 => ((s.clip() + 1.0) * const { u32::ORIGIN as f64 }) as u32,
    u64 => ((s.clip() + 1.0) * const { u64::ORIGIN as f64 }) as u64,

    // float
    f32 => s as f32,
    f64 => s
);

#[cfg(test)]
mod tests {
    macro_rules! impl_test {
        ($name:ident, $from:ty, $into:ty) => {
            #[test]
            fn $name() {
                assert_eq!(
                    <$into as FromSample::<_>>::from_sample(<$from>::ORIGIN),
                    <$into as Sample>::ORIGIN,
                    "origin"
                );

                assert_eq!(
                    <$into as FromSample::<_>>::from_sample(<$from>::RANGE.0),
                    <$into as ClipSample>::RANGE.0,
                    "lower limit"
                );

                let mut tolerance = 0 as $into;
                if size_of::<$from>() < size_of::<$into>() {
                    let from_range = <$from as ClipSample>::RANGE.1 - <$from>::ORIGIN;
                    let into_range = <$into as ClipSample>::RANGE.1 - <$into>::ORIGIN;
                    tolerance = into_range / from_range as $into;
                }

                let upper_result = <$into as FromSample<_>>::from_sample(<$from>::RANGE.1);

                const UPPER_LIMIT: $into = <$into as ClipSample>::RANGE.1;
                let upper_diff = UPPER_LIMIT - upper_result;

                assert!(
                    upper_diff <= tolerance,
                    "limit: {UPPER_LIMIT}, result: {upper_result}, tolerance: {tolerance}, diff: {upper_diff}"
                )
            }
        };
        ($name:ident, $from:ident) => {
            mod $name {
                use crate::ops::{ClipSample, FromSample};
                use phonic_signal::Sample;

                impl_test!(into_i8, $from, i8);
                impl_test!(into_i16, $from, i16);
                impl_test!(into_i32, $from, i32);
                impl_test!(into_i64, $from, i64);

                impl_test!(into_u8, $from, u8);
                impl_test!(into_u16, $from, u16);
                impl_test!(into_u32, $from, u32);
                impl_test!(into_u64, $from, u64);

                impl_test!(into_f32, $from, f32);
                impl_test!(into_f64, $from, f64);
            }
        };
    }

    impl_test!(from_i8, i8);
    impl_test!(from_i16, i16);
    impl_test!(from_i32, i32);
    impl_test!(from_i64, i64);

    impl_test!(from_u8, u8);
    impl_test!(from_u16, u16);
    impl_test!(from_u32, u32);
    impl_test!(from_u64, u64);

    impl_test!(from_f32, f32);
    impl_test!(from_f64, f64);
}

use crate::ops::{FromSample, IntoSample};
use num_traits::Inv;
use phonic_macro::impl_deref_signal;
use phonic_signal::{PhonicResult, Sample, SignalReader};
use std::ops::{Mul, Neg};

pub struct Gain<T, R> {
    inner: T,
    ratio: R,
}

pub trait GainSample: Sample {
    type Ratio;

    fn gain(self, ratio: Self::Ratio) -> Self;
}

pub trait DbRatio {
    fn db_ratio(self) -> Self;
}

impl<T, R> Gain<T, R> {
    pub fn new(inner: T, ratio: R) -> Self {
        Self { inner, ratio }
    }

    pub fn new_db(inner: T, db: R) -> Self
    where
        R: DbRatio,
    {
        Self::new(inner, db.db_ratio())
    }

    pub fn attenuate(inner: T, ratio: R) -> Self
    where
        R: Inv<Output = R>,
    {
        Self::new(inner, ratio.inv())
    }

    pub fn attenuate_db(inner: T, db: R) -> Self
    where
        R: DbRatio + Inv<Output = R>,
    {
        Self::attenuate(inner, db.db_ratio())
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl_deref_signal! {
    impl<T, R> _ + !SignalReader + !SignalWriter for Gain<T, R> {
        type Target = T;

        &self -> &self.inner;
        &mut self -> &mut self.inner;
    }
}

impl<T, R> SignalReader for Gain<T, R>
where
    T: SignalReader,
    T::Sample: GainSample<Ratio = R>,
    R: Copy,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize> {
        let n = self.inner.read(buf)?;
        buf[..n].iter_mut().for_each(|s| *s = s.gain(self.ratio));

        Ok(n)
    }
}

macro_rules! impl_gain {
    ($sample:ident as $ratio_ty:ty; |$self:ident, $ratio:ident| $func:expr) => {
        impl GainSample for $sample {
            type Ratio = $ratio_ty;

            #[inline]
            fn gain($self, $ratio: Self::Ratio) -> Self {
                $func
            }
        }
    };
    ($sample:ident as $ratio_ty:ty) => {
        impl_gain!($sample as $ratio_ty; |self, r| {
            <$ratio_ty>::from_sample(self).gain(r).into_sample()
        });
    }
}

impl_gain!(i8 as f32);
impl_gain!(i16 as f32);
impl_gain!(i32 as f64);
impl_gain!(i64 as f64);

impl_gain!(u8 as f32);
impl_gain!(u16 as f32);
impl_gain!(u32 as f64);
impl_gain!(u64 as f64);

impl_gain!(f32 as f32; |self, r| self.mul(r));
impl_gain!(f64 as f64; |self, r| self.mul(r));

macro_rules! impl_db_ratio {
    ($ratio:ty) => {
        impl DbRatio for $ratio {
            fn db_ratio(self) -> Self {
                <$ratio>::powf(10.0, self / 20.0)
            }
        }
    };
}

impl_db_ratio!(f32);
impl_db_ratio!(f64);

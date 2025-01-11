use phonic_signal::{delegate_signal, PhonicResult, Sample, SignalExt, SignalReader};
use std::{mem::MaybeUninit, ops::Neg};

pub trait ComplementSample: Sample {
    fn complement(self) -> Self;
}

pub struct Complement<T> {
    inner: T,
}

impl<T> Complement<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

delegate_signal! {
    impl<T> * + !Read + !Write for Complement<T> {
        Self as T;

        &self => &self.inner;
        &mut self => &mut self.inner;
    }
}

impl<T> SignalReader for Complement<T>
where
    T: SignalReader,
    T::Sample: ComplementSample,
{
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let samples = self.inner.read_init(buf)?;
        samples.iter_mut().for_each(|s| *s = s.complement());

        Ok(samples.len())
    }
}

macro_rules! impl_complement {
    ($sample:ident, $self:ident, $func:expr) => {
        impl ComplementSample for $sample {
            #[inline]
            fn complement($self) -> Self {
                $func
            }
        }
    };
}

macro_rules! impl_unsigned_complement {
    ($sample:ident, $self:ident) => {
        impl_complement!($sample, $self, {
            if $self >= $sample::ORIGIN {
                $sample::ORIGIN - ($self - $sample::ORIGIN)
            } else {
                $sample::ORIGIN.saturating_add($sample::ORIGIN - $self)
            }
        });
    };
}

impl_complement!(i8, self, self.saturating_neg());
impl_complement!(i16, self, self.saturating_neg());
impl_complement!(i32, self, self.saturating_neg());
impl_complement!(i64, self, self.saturating_neg());

impl_unsigned_complement!(u8, self);
impl_unsigned_complement!(u16, self);
impl_unsigned_complement!(u32, self);
impl_unsigned_complement!(u64, self);

impl_complement!(f32, self, self.neg());
impl_complement!(f64, self, self.neg());

pub trait Sample: Copy + Sized + Send + Sync + 'static {
    const ORIGIN: Self;
}

macro_rules! impl_int_sample {
    ($s:ty) => {
        impl Sample for $s {
            const ORIGIN: Self = 0;
        }
    };
}

macro_rules! impl_uint_sample {
    ($s:ty) => {
        impl Sample for $s {
            const ORIGIN: Self = Self::MAX / 2 + 1;
        }
    };
}

macro_rules! impl_float_sample {
    ($s:ty) => {
        impl Sample for $s {
            const ORIGIN: Self = 0.0;
        }
    };
}

impl_int_sample!(i8);
impl_int_sample!(i16);
impl_int_sample!(i32);
impl_int_sample!(i64);

impl_uint_sample!(u8);
impl_uint_sample!(u16);
impl_uint_sample!(u32);
impl_uint_sample!(u64);

impl_float_sample!(f32);
impl_float_sample!(f64);

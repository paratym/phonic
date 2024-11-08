pub trait Sample: Copy + Sized + Send + Sync + 'static {
    const ORIGIN: Self;
    const RANGE: (Self, Self);

    fn clamped(self) -> Self
    where
        Self: PartialOrd,
    {
        if self < Self::RANGE.0 {
            Self::RANGE.0
        } else if self > Self::RANGE.1 {
            Self::RANGE.1
        } else {
            self
        }
    }
}

macro_rules! impl_int_sample {
    ($s:ty, $t: ident) => {
        impl Sample for $s {
            const ORIGIN: Self = 0;
            const RANGE: (Self, Self) = (Self::MIN, Self::MAX);
        }
    };
}

macro_rules! impl_uint_sample {
    ($s:ty, $t:ident) => {
        impl Sample for $s {
            const ORIGIN: Self = Self::MAX / 2;
            const RANGE: (Self, Self) = (Self::MIN, Self::MAX);
        }
    };
}

macro_rules! impl_float_sample {
    ($s:ty, $t:ident) => {
        impl Sample for $s {
            const ORIGIN: Self = 0.0;
            const RANGE: (Self, Self) = (-1.0, 1.0);
        }
    };
}

impl_int_sample!(i8, I8);
impl_int_sample!(i16, I16);
impl_int_sample!(i32, I32);
impl_int_sample!(i64, I64);

impl_uint_sample!(u8, U8);
impl_uint_sample!(u16, U16);
impl_uint_sample!(u32, U32);
impl_uint_sample!(u64, U64);

impl_float_sample!(f32, F32);
impl_float_sample!(f64, F64);

pub trait FromSample<S: Sample> {
    fn from_sample(sample: S) -> Self;
}

pub trait IntoSample<S: Sample> {
    fn into_sample(self) -> S;
}

impl<T: Sample, S: Sample + FromSample<T>> IntoSample<S> for T {
    #[inline(always)]
    fn into_sample(self) -> S {
        S::from_sample(self)
    }
}

macro_rules! impl_convert {
    ($from:ty, $to:ty, $sample:ident, $func:expr) => {
        impl FromSample<$from> for $to {
            #[inline]
            fn from_sample($sample: $from) -> Self {
                $func
            }
        }
    };
}

// u8 to ...
impl_convert!(u8, u8, s, s);
impl_convert!(u8, u16, s, (s as u16) << 8);
impl_convert!(u8, u32, s, (s as u32) << 24);
impl_convert!(u8, u64, s, todo!());

impl_convert!(u8, i8, s, s.wrapping_sub(0x80) as i8);
impl_convert!(u8, i16, s, ((s.wrapping_sub(0x80) as i8) as i16) << 8);
impl_convert!(u8, i32, s, ((s.wrapping_sub(0x80) as i8) as i32) << 24);
impl_convert!(u8, i64, s, todo!());

impl_convert!(u8, f32, s, ((s as f32) / 128.0) - 1.0);
impl_convert!(u8, f64, s, ((s as f64) / 128.0) - 1.0);

// u16 to ...
impl_convert!(u16, u8, s, (s >> 8) as u8);
impl_convert!(u16, u16, s, s);
impl_convert!(u16, u32, s, (s as u32) << 16);
impl_convert!(u16, u64, s, todo!());

impl_convert!(u16, i8, s, (s.wrapping_sub(0x8000) >> 8) as i8);
impl_convert!(u16, i16, s, s.wrapping_sub(0x8000) as i16);
impl_convert!(u16, i32, s, ((s.wrapping_sub(0x8000) as i16) as i32) << 16);
impl_convert!(u16, i64, s, todo!());

impl_convert!(u16, f32, s, ((s as f32) / 32_768.0) - 1.0);
impl_convert!(u16, f64, s, ((s as f64) / 32_768.0) - 1.0);

// u32 to ...
impl_convert!(u32, u8, s, (s >> 24) as u8);
impl_convert!(u32, u16, s, (s >> 16) as u16);
impl_convert!(u32, u32, s, s);
impl_convert!(u32, u64, s, todo!());

impl_convert!(u32, i8, s, (s.wrapping_sub(0x8000_0000) >> 24) as i8);
impl_convert!(u32, i16, s, (s.wrapping_sub(0x8000_0000) >> 16) as i16);
impl_convert!(u32, i32, s, s.wrapping_sub(0x8000_0000) as i32);
impl_convert!(u32, i64, s, todo!());

impl_convert!(u32, f32, s, (((s as f64) / 2_147_483_648.0) - 1.0) as f32);
impl_convert!(u32, f64, s, ((s as f64) / 2_147_483_648.0) - 1.0);

// u64 to ...
impl_convert!(u64, u8, s, todo!());
impl_convert!(u64, u16, s, todo!());
impl_convert!(u64, u32, s, todo!());
impl_convert!(u64, u64, s, s);

impl_convert!(u64, i8, s, todo!());
impl_convert!(u64, i16, s, todo!());
impl_convert!(u64, i32, s, todo!());
impl_convert!(u64, i64, s, todo!());

impl_convert!(u64, f32, s, todo!());
impl_convert!(u64, f64, s, todo!());

// i8 to ...
#[inline(always)]
fn i8_to_u8(s: i8) -> u8 {
    (s as u8).wrapping_add(0x80)
}

impl_convert!(i8, u8, s, i8_to_u8(s));
impl_convert!(i8, u16, s, (i8_to_u8(s) as u16) << 8);
impl_convert!(i8, u32, s, (i8_to_u8(s) as u32) << 24);
impl_convert!(i8, u64, s, todo!());

impl_convert!(i8, i8, s, s);
impl_convert!(i8, i16, s, (s as i16) << 8);
impl_convert!(i8, i32, s, (s as i32) << 24);
impl_convert!(i8, i64, s, todo!());

impl_convert!(i8, f32, s, s as f32 / 128.0);
impl_convert!(i8, f64, s, s as f64 / 128.0);

// i16 to ...
#[inline(always)]
fn i16_to_u16(s: i16) -> u16 {
    (s as u16).wrapping_add(0x8000)
}

impl_convert!(i16, u8, s, (i16_to_u16(s) >> 8) as u8);
impl_convert!(i16, u16, s, i16_to_u16(s));
impl_convert!(i16, u32, s, (i16_to_u16(s) as u32) << 16);
impl_convert!(i16, u64, s, todo!());

impl_convert!(i16, i8, s, (s >> 8) as i8);
impl_convert!(i16, i16, s, s);
impl_convert!(i16, i32, s, (s as i32) << 16);
impl_convert!(i16, i64, s, todo!());

impl_convert!(i16, f32, s, s as f32 / 32_768.0);
impl_convert!(i16, f64, s, s as f64 / 32_768.0);

// i32 to ...
#[inline(always)]
fn i32_to_u32(s: i32) -> u32 {
    (s as u32).wrapping_add(0x8000_0000)
}

impl_convert!(i32, u8, s, (i32_to_u32(s) >> 24) as u8);
impl_convert!(i32, u16, s, (i32_to_u32(s) >> 16) as u16);
impl_convert!(i32, u32, s, i32_to_u32(s));
impl_convert!(i32, u64, s, todo!());

impl_convert!(i32, i8, s, (s >> 24) as i8);
impl_convert!(i32, i16, s, (s >> 16) as i16);
impl_convert!(i32, i32, s, s);
impl_convert!(i32, i64, s, todo!());

impl_convert!(i32, f32, s, (s as f64 / 2_147_483_648.0) as f32);
impl_convert!(i32, f64, s, s as f64 / 2_147_483_648.0);

// i64 to ...
impl_convert!(i64, u8, s, todo!());
impl_convert!(i64, u16, s, todo!());
impl_convert!(i64, u32, s, todo!());
impl_convert!(i64, u64, s, todo!());

impl_convert!(i64, i8, s, todo!());
impl_convert!(i64, i16, s, todo!());
impl_convert!(i64, i32, s, todo!());
impl_convert!(i64, i64, s, s);

impl_convert!(i64, f32, s, todo!());
impl_convert!(i64, f64, s, todo!());

// f32 to ...
impl_convert!(f32, u8, s, ((s.clamped() + 1.0) * 128.0) as u8);
impl_convert!(f32, u16, s, ((s.clamped() + 1.0) * 32_768.0) as u16);
impl_convert!(
    f32,
    u32,
    s,
    ((s.clamped() + 1.0) as f64 * 2_147_483_648.0) as u32
);
impl_convert!(
    f32,
    u64,
    s,
    (s.clamped() as f64 * 9_223_372_036_854_775_808.0) as u64
);

impl_convert!(f32, i8, s, (s.clamped() * 128.0) as i8);
impl_convert!(f32, i16, s, (s.clamped() * 32_768.0) as i16);
impl_convert!(f32, i32, s, (s.clamped() as f64 * 2_147_483_648.0) as i32);
impl_convert!(
    f32,
    i64,
    s,
    (s.clamped() as f64 * 9_223_372_036_854_775_808.0) as i64
);

impl_convert!(f32, f32, s, s);
impl_convert!(f32, f64, s, s as f64);

// f64 to ...
impl_convert!(f64, u8, s, ((s.clamped() + 1.0) * 128.0) as u8);
impl_convert!(f64, u16, s, ((s.clamped() + 1.0) * 32_768.0) as u16);
impl_convert!(f64, u32, s, ((s.clamped() + 1.0) * 2_147_483_648.0) as u32);
impl_convert!(
    f64,
    u64,
    s,
    ((s.clamped() + 1.0) * 9_223_372_036_854_775_808.0) as u64
);

impl_convert!(f64, i8, s, (s.clamped() * 128.0) as i8);
impl_convert!(f64, i16, s, (s.clamped() * 32_768.0) as i16);
impl_convert!(f64, i32, s, (s.clamped() * 2_147_483_648.0) as i32);
impl_convert!(
    f64,
    i64,
    s,
    (s.clamped() * 9_223_372_036_854_775_808.0) as i64
);

impl_convert!(f64, f32, s, s as f32);
impl_convert!(f64, f64, s, s);

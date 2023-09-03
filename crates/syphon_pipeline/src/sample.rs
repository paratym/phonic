use std::{
    mem::size_of,
    ops::{Add, Div, Mul, Sub},
};

#[derive(Copy, Clone, Eq)]
pub enum Endianess {
    Big,
    Little,
    Native,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum SampleFormat {
    Signed(Endianess),
    Unsigned(Endianess),
    Float(Endianess),
}

pub trait Sample:
    Copy
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + PartialOrd
    + PartialEq
    + Sized
{
    const N_BYTES: usize;
    const FORMAT: SampleFormat;

    const MIN: Self;
    const MID: Self;
    const MAX: Self;

    fn clamped(self) -> Self;

    #[inline(always)]
    fn range() -> Self {
        Self::MAX - Self::MIN
    }
}

macro_rules! impl_int_sample {
    ($s:ty) => {
        impl Sample for $s {
            const N_BYTES: usize = size_of::<$s>();
            const FORMAT: SampleFormat = SampleFormat::Signed(Endianess::Native);

            const MIN: Self = <$s>::MIN;
            const MID: Self = 0;
            const MAX: Self = <$s>::MAX;

            #[inline(always)]
            fn clamped(self) -> Self {
                self
            }
        }
    };
}

macro_rules! impl_uint_sample {
    ($s:ty) => {
        impl Sample for $s {
            const N_BYTES: usize = size_of::<$s>();
            const FORMAT: SampleFormat = SampleFormat::Unsigned(Endianess::Native);

            const MIN: Self = <$s>::MIN;
            const MID: Self = Self::MAX / 2;
            const MAX: Self = <$s>::MAX;

            #[inline(always)]
            fn clamped(self) -> Self {
                self
            }
        }
    };
}

macro_rules! impl_float_sample {
    ($s:ty) => {
        impl Sample for $s {
            const N_BYTES: usize = size_of::<$s>();
            const FORMAT: SampleFormat = SampleFormat::Float(Endianess::Native);

            const MIN: Self = -1.0;
            const MID: Self = 0.0;
            const MAX: Self = 1.0;

            #[inline]
            fn clamped(self) -> Self {
                if self > Self::MAX {
                    Self::MAX
                } else if self < Self::MIN {
                    Self::MIN
                } else {
                    self
                }
            }
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

impl Endianess {
    pub fn is_big(&self) -> bool {
        match self {
            Self::Big => true,
            Self::Native if cfg!(endianess = "big") => true,
            _ => false,
        }
    }

    pub fn is_little(&self) -> bool {
        match self {
            Self::Little => true,
            Self::Native if cfg!(endianess = "little") => true,
            _ => false,
        }
    }

    pub fn is_native(&self) -> bool {
        match self {
            Self::Native => true,
            Self::Big if cfg!(endianess = "big") => true,
            Self::Little if cfg!(endianess = "little") => true,
            _ => false,
        }
    }
}

impl PartialEq for Endianess {
    fn eq(&self, other: &Self) -> bool {
        (self.is_native() && other.is_native())
            || (self.is_big() && other.is_big())
            || (self.is_little() && other.is_little())
    }
}

impl SampleFormat {
    pub fn endianess(&self) -> Endianess {
        match self {
            Self::Signed(e) => *e,
            Self::Unsigned(e) => *e,
            Self::Float(e) => *e,
        }
    }

    pub fn endianess_mut(&mut self) -> &mut Endianess {
        match self {
            Self::Signed(e) => e,
            Self::Unsigned(e) => e,
            Self::Float(e) => e,
        }
    }

    pub fn layout_eq(&self, other: &Self) -> bool {
        match self {
            Self::Signed(_) if matches!(other, Self::Signed(_)) => true,
            Self::Unsigned(_) if matches!(other, Self::Unsigned(_)) => true,
            Self::Float(_) if matches!(other, Self::Float(_)) => true,
            _ => false,
        }
    }
}

pub trait FromSample<S: Sample>: Sample {
    fn from(sample: S) -> Self;
}

#[inline(always)]
fn i8_to_u8(s: i8) -> u8 {
    (s as u8).wrapping_add(0x80)
}

macro_rules! impl_convert {
    ($from:ty, $to:ty, $sample:ident, $func:expr) => {
        impl FromSample<$from> for $to {
            #[inline(always)]
            fn from($sample: $from) -> Self {
                $func
            }
        }
    };
}

impl_convert!(i8, u8, s, i8_to_u8(s)); // u8
impl_convert!(i8, u16, s, (i8_to_u8(s) as u16) << 8); // u16
// impl_convert!(i8, u24, s, u24::from((i8_to_u8(s) as u32) << 16)); // u24
impl_convert!(i8, u32, s, (i8_to_u8(s) as u32) << 24); // u3

impl_convert!(i8, i8, s, s); // i8
impl_convert!(i8, i16, s, (s as i16) << 8); // i16
// impl_convert!(i8, i24, s, i24::from((s as i32) << 16)); // i24
impl_convert!(i8, i32, s, (s as i32) << 24); // i32

impl_convert!(i8, f32, s, s as f32 / 128.0); // f32
impl_convert!(i8, f64, s, s as f64 / 128.0); // f64

// i16 to ...

#[inline(always)]
fn i16_to_u16(s: i16) -> u16 {
    (s as u16).wrapping_add(0x8000)
}

impl_convert!(i16, u8, s, (i16_to_u16(s) >> 8) as u8); // u8
impl_convert!(i16, u16, s, i16_to_u16(s)); // u16
// impl_convert!(i16, u24, s, u24::from((i16_to_u16(s) as u32) << 8)); // u24
impl_convert!(i16, u32, s, (i16_to_u16(s) as u32) << 16); // u32

impl_convert!(i16, i8, s, (s >> 8) as i8); // i8
impl_convert!(i16, i16, s, s); // i16
// impl_convert!(i16, i24, s, i24::from((s as i32) << 8)); // i24
impl_convert!(i16, i32, s, (s as i32) << 16); // i32

impl_convert!(i16, f32, s, s as f32 / 32_768.0); // f32
impl_convert!(i16, f64, s, s as f64 / 32_768.0); // f64

// i24 to ...

// #[inline(always)]
// fn i24_to_u32(s: i24) -> u32 {
//     ((s.clamped().inner() << 8) as u32).wrapping_add(0x8000_0000)
// }

// impl_convert!(i24, u8, s, (i24_to_u32(s) >> 24) as u8); // u8
// impl_convert!(i24, u16, s, (i24_to_u32(s) >> 16) as u16); // u16
// impl_convert!(i24, u24, s, u24::from(i24_to_u32(s) >> 8)); // u24
// impl_convert!(i24, u32, s, i24_to_u32(s)); // u32

// impl_convert!(i24, i8, s, (s.clamped().inner() >> 16) as i8); // i8
// impl_convert!(i24, i16, s, (s.clamped().inner() >> 8) as i16); // i16
// impl_convert!(i24, i24, s, s); // i24
// impl_convert!(i24, i32, s, (s.clamped().inner()) << 8); // i32

// impl_convert!(i24, f32, s, s.clamped().inner() as f32 / 8_388_608.0); // f32
// impl_convert!(i24, f64, s, s.clamped().inner() as f64 / 8_388_608.0); // f64

// i32 to ...

#[inline(always)]
fn i32_to_u32(s: i32) -> u32 {
    (s as u32).wrapping_add(0x8000_0000)
}

impl_convert!(i32, u8, s, (i32_to_u32(s) >> 24) as u8); // u8
impl_convert!(i32, u16, s, (i32_to_u32(s) >> 16) as u16); // u16
// impl_convert!(i32, u24, s, u24::from(i32_to_u32(s) >> 8)); // u24
impl_convert!(i32, u32, s, i32_to_u32(s)); // u32

impl_convert!(i32, i8, s, (s >> 24) as i8); // i8
impl_convert!(i32, i16, s, (s >> 16) as i16); // i16
// impl_convert!(i32, i24, s, i24::from(s >> 8)); // i24
impl_convert!(i32, i32, s, s); // i32

impl_convert!(i32, f32, s, (s as f64 / 2_147_483_648.0) as f32); // f32
impl_convert!(i32, f64, s, s as f64 / 2_147_483_648.0); // f64

// u8 to ...

impl_convert!(u8, u8, s, s); // u8
impl_convert!(u8, u16, s, (s as u16) << 8); // u16
// impl_convert!(u8, u24, s, u24::from((s as u32) << 16)); // u24
impl_convert!(u8, u32, s, (s as u32) << 24); // u32

impl_convert!(u8, i8, s, s.wrapping_sub(0x80) as i8); // i8
impl_convert!(u8, i16, s, ((s.wrapping_sub(0x80) as i8) as i16) << 8); // i16
// impl_convert!(u8, i24, s, i24::from(((s.wrapping_sub(0x80) as i8) as i32) << 16)); // i24
impl_convert!(u8, i32, s, ((s.wrapping_sub(0x80) as i8) as i32) << 24); // i32

impl_convert!(u8, f32, s, ((s as f32) / 128.0) - 1.0); // f32
impl_convert!(u8, f64, s, ((s as f64) / 128.0) - 1.0); // f64

// u16 to ...

impl_convert!(u16, u8, s, (s >> 8) as u8); // u8
impl_convert!(u16, u16, s, s); // u16
// impl_convert!(u16, u24, s, u24::from((s as u32) << 8)); // u24
impl_convert!(u16, u32, s, (s as u32) << 16); // u32

impl_convert!(u16, i8, s, (s.wrapping_sub(0x8000) >> 8) as i8); // i8
impl_convert!(u16, i16, s, s.wrapping_sub(0x8000) as i16); // i16
// impl_convert!(u16, i24, s, i24::from(((s.wrapping_sub(0x8000) as i16) as i32) << 8)); // i24
impl_convert!(u16, i32, s, ((s.wrapping_sub(0x8000) as i16) as i32) << 16); // i32

impl_convert!(u16, f32, s, ((s as f32) / 32_768.0) - 1.0); // f32
impl_convert!(u16, f64, s, ((s as f64) / 32_768.0) - 1.0); // f64

// u24 to ...

// impl_convert!(u24, u8, s, (s.clamped().inner() >> 16) as u8); // u8
// impl_convert!(u24, u16, s, (s.clamped().inner() >> 8) as u16); // u16
// impl_convert!(u24, u24, s, s); // u24
// impl_convert!(u24, u32, s, s.clamped().inner() << 8); // u32

// impl_convert!(u24, i8, s, (s.clamped().inner().wrapping_sub(0x80_0000) >> 16) as i8); // i8
// impl_convert!(u24, i16, s, (s.clamped().inner().wrapping_sub(0x80_0000) >> 8) as i16); // i16
// impl_convert!(u24, i24, s, i24::from(s.clamped().inner().wrapping_sub(0x80_0000) as i32)); // i24
// impl_convert!(u24, i32, s, (s.clamped().inner().wrapping_sub(0x80_0000) << 8) as i32); // i32

// impl_convert!(u24, f32, s, ((s.clamped().inner() as f32) / 8_388_608.0) - 1.0); // f32
// impl_convert!(u24, f64, s, ((s.clamped().inner() as f64) / 8_388_608.0) - 1.0); // f64

// u32 to ...

impl_convert!(u32, u8, s, (s >> 24) as u8); // u8
impl_convert!(u32, u16, s, (s >> 16) as u16); // u16
// impl_convert!(u32, u24, s, u24::from(s >> 8)); // u24
impl_convert!(u32, u32, s, s); // u32

impl_convert!(u32, i8, s, (s.wrapping_sub(0x8000_0000) >> 24) as i8); // i8
impl_convert!(u32, i16, s, (s.wrapping_sub(0x8000_0000) >> 16) as i16); // i16
// impl_convert!(u32, i24, s, i24::from((s.wrapping_sub(0x8000_0000) as i32) >> 8)); // i24
impl_convert!(u32, i32, s, s.wrapping_sub(0x8000_0000) as i32); // i32

impl_convert!(u32, f32, s, (((s as f64) / 2_147_483_648.0) - 1.0) as f32); // f32
impl_convert!(u32, f64, s, ((s as f64) / 2_147_483_648.0) - 1.0); // f64

// f32 to ...

impl_convert!(f32, u8, s, ((s.clamped() + 1.0) * 128.0) as u8); // u8
impl_convert!(f32, u16, s, ((s.clamped() + 1.0) * 32_768.0) as u16); // u16
// impl_convert!(f32, u24, s, u24::from(((s.clamped() + 1.0) * 8_388_608.0) as u32)); // u24
impl_convert!(f32, u32, s, ((s.clamped() + 1.0) as f64 * 2_147_483_648.0) as u32); // u32

impl_convert!(f32, i8, s, (s.clamped() * 128.0) as i8); // i8
impl_convert!(f32, i16, s, (s.clamped() * 32_768.0) as i16); // i16
// impl_convert!(f32, i24, s, i24::from((s.clamped() * 8_388_608.0) as i32)); // i24
impl_convert!(f32, i32, s, (s.clamped() as f64 * 2_147_483_648.0) as i32); // i32

impl_convert!(f32, f32, s, s); // f32
impl_convert!(f32, f64, s, s as f64); // f64

// f64 to ...

impl_convert!(f64, u8, s, ((s.clamped() + 1.0) * 128.0) as u8); // u8
impl_convert!(f64, u16, s, ((s.clamped() + 1.0) * 32_768.0) as u16); // u16
// impl_convert!(f64, u24, s, u24::from(((s.clamped() + 1.0) * 8_388_608.0) as u32)); // u24
impl_convert!(f64, u32, s, ((s.clamped() + 1.0) * 2_147_483_648.0) as u32); // u32

impl_convert!(f64, i8, s, (s.clamped() * 128.0) as i8); // i8
impl_convert!(f64, i16, s, (s.clamped() * 32_768.0) as i16); // i16
// impl_convert!(f64, i24, s, i24::from((s.clamped() * 8_388_608.0) as i32)); // i24
impl_convert!(f64, i32, s, (s.clamped() * 2_147_483_648.0) as i32); // i32

impl_convert!(f64, f32, s, s as f32); // f32
impl_convert!(f64, f64, s, s); // f64


pub trait ConvertibleSample:
    Sample
    + FromSample<i8>
    + FromSample<u8>
    + FromSample<i16>
    + FromSample<u16>
    // + FromSample<i24>
    // + FromSample<u24>
    + FromSample<i32>
    + FromSample<u32>
    + FromSample<f32>
    + FromSample<f64>
{
}

impl<S> ConvertibleSample for S where
    S: Sample
        + FromSample<i8>
        + FromSample<u8>
        + FromSample<i16>
        + FromSample<u16>
        // + FromSample<i24>
        // + FromSample<u24>
        + FromSample<i32>
        + FromSample<u32>
        + FromSample<f32>
        + FromSample<f64>
{
}

// pub struct SampleBuffer<'a, S: Sample> {
//     sample_spec: SampleSpec,
//     pub buf: &'a mut [S],
// }

// impl<'a, S: Sample> SampleBuffer<'a, S> {
//     pub fn new(buf: &'a mut [S], spec: SampleSpec) -> Self {
//         let buf_frames = (buf.len() / spec.n_channels as usize) as u64;
//         let n_frames = spec.n_frames.unwrap_or(buf_frames).min(buf_frames);
//         let mut sample_format = S::FORMAT.clone();
//         *sample_format.endianess_mut() = spec.sample_format.endianess();

//         Self {
//             buf,
//             sample_spec: SampleSpec {
//                 n_frames: Some(n_frames),
//                 bits_per_sample: S::N_BYTES as u16 * 8,
//                 sample_format,
//                 ..spec
//             },
//         }
//     }

//     pub fn sample_spec(&self) -> &SampleSpec {
//         &self.sample_spec
//     }
// }

// impl<S: Sample> SampleBuffer<'_, S> {
//     pub fn read_from(&mut self, reader: &mut dyn SampleReader<S>) -> Result<usize, AudioIoErr> {
//         self.sample_spec.try_copy_from(reader.sample_spec())?;
//         return match reader.read(&mut self.buf) {
//             Ok(n) if n % self.sample_spec.n_channels as usize == 0 => Ok(n),
//             Ok(_) => Err(AudioIoErr::BlockSizeMismatch),
//             Err(e) => Err(e.into()),
//         };
//     }

//     pub fn write_from(&mut self, reader: &mut dyn SampleReader<S>) -> Result<usize, AudioIoErr> {
//         self.sample_spec.try_copy_from(reader.sample_spec())?;
//         return match reader.read(&mut self.buf) {
//             Ok(n) if n % self.sample_spec.n_channels as usize == 0 => Ok(n),
//             Ok(_) => Err(AudioIoErr::BlockSizeMismatch),
//             Err(e) => Err(e.into()),
//         };
//     }
// }

// macro_rules! impl_sample_buffer {
//     ($s:ty) => {
//         pub fn to_ne(&mut self) {
//             if self.sample_spec.sample_format.endianess() == Endianess::Native {
//                 return;
//             }

//             if cfg!(endianess = "little") {
//                 self.to_le();
//             } else {
//                 self.to_be();
//             }
//         }
//     };
// }

// macro_rules! impl_int_sample_buffer {
//     ($s:ty) => {
//         impl<'a> SampleBuffer<'a, $s> {
//             impl_sample_buffer!($s);

//             pub fn to_be(&mut self) {
//                 if self.sample_spec.sample_format.endianess() == Endianess::Big {
//                     return;
//                 }

//                 *self.sample_spec.sample_format.endianess_mut() = Endianess::Big;
//                 for sample in self.buf.iter_mut() {
//                     *sample = sample.to_be();
//                 }
//             }

//             pub fn to_le(&mut self) {
//                 if self.sample_spec.sample_format.endianess() == Endianess::Little {
//                     return;
//                 }

//                 *self.sample_spec.sample_format.endianess_mut() = Endianess::Little;
//                 for sample in self.buf.iter_mut() {
//                     *sample = sample.to_le();
//                 }
//             }
//         }
//     };
// }

// macro_rules! impl_float_sample_buffer {
//     ($s:ty) => {
//         impl<'a> SampleBuffer<'a, $s> {
//             impl_sample_buffer!($s);

//             pub fn to_be(&mut self) {
//                 if self.sample_spec.sample_format.endianess() == Endianess::Big {
//                     return;
//                 }

//                 *self.sample_spec.sample_format.endianess_mut() = Endianess::Big;
//                 for sample in self.buf.iter_mut() {
//                     *sample = <$s>::from_bits(sample.to_bits().to_be());
//                 }
//             }

//             pub fn to_le(&mut self) {
//                 if self.sample_spec.sample_format.endianess() == Endianess::Little {
//                     return;
//                 }

//                 *self.sample_spec.sample_format.endianess_mut() = Endianess::Little;
//                 for sample in self.buf.iter_mut() {
//                     *sample = <$s>::from_bits(sample.to_bits().to_le());
//                 }
//             }
//         }
//     };
// }

// impl_int_sample_buffer!(u8);
// impl_int_sample_buffer!(u16);
// impl_int_sample_buffer!(u32);
// impl_int_sample_buffer!(u64);

// impl_int_sample_buffer!(i8);
// impl_int_sample_buffer!(i16);
// impl_int_sample_buffer!(i32);
// impl_int_sample_buffer!(i64);

// impl_float_sample_buffer!(f32);
// impl_float_sample_buffer!(f64);

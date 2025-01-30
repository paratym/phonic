use crate::{CodecTag, StreamSpec};
use phonic_signal::utils::{FromDuration, NFrames, NSamples};
use std::{
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    time::Duration,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NBytes {
    pub n_bytes: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NBlocks {
    pub n_blocks: u64,
}

pub trait FromStreamDuration<T> {
    fn from_stream_duration<C: CodecTag>(duration: T, spec: &StreamSpec<C>) -> Self;
}

pub trait IntoStreamDuration<T> {
    fn into_stream_duration<C: CodecTag>(self, spec: &StreamSpec<C>) -> T;
}

macro_rules! impl_ops {
    ($unit:ty, $inner:ident) => {
        impl From<u64> for $unit {
            fn from($inner: u64) -> Self {
                Self { $inner }
            }
        }

        impl Add<$unit> for $unit {
            type Output = Self;

            fn add(self, rhs: Self) -> Self {
                let inner = self.$inner.add(rhs.$inner);
                Self::from(inner)
            }
        }

        impl AddAssign<$unit> for $unit {
            fn add_assign(&mut self, rhs: Self) {
                self.$inner += rhs.$inner
            }
        }

        impl Sub<$unit> for $unit {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self {
                let inner = self.$inner.sub(rhs.$inner);
                Self::from(inner)
            }
        }

        impl SubAssign<$unit> for $unit {
            fn sub_assign(&mut self, rhs: Self) {
                self.$inner -= rhs.$inner
            }
        }

        impl Mul<u64> for $unit {
            type Output = Self;

            fn mul(self, rhs: u64) -> Self {
                let inner = self.$inner.mul(rhs);
                Self::from(inner)
            }
        }

        impl MulAssign<u64> for $unit {
            fn mul_assign(&mut self, rhs: u64) {
                self.$inner *= rhs
            }
        }

        impl Div<$unit> for $unit {
            type Output = u64;

            fn div(self, rhs: Self) -> u64 {
                self.$inner.div(rhs.$inner)
            }
        }

        impl Div<u64> for $unit {
            type Output = Self;

            fn div(self, rhs: u64) -> Self {
                let inner = self.$inner.div(rhs);
                Self::from(inner)
            }
        }

        impl DivAssign<u64> for $unit {
            fn div_assign(&mut self, rhs: u64) {
                self.$inner /= rhs
            }
        }
    };
}

impl_ops!(NBytes, n_bytes);
impl_ops!(NBlocks, n_blocks);

impl<T: FromDuration<U>, U> FromStreamDuration<U> for T {
    fn from_stream_duration<C: CodecTag>(duration: U, spec: &StreamSpec<C>) -> Self {
        Self::from_duration(duration, &spec.decoded)
    }
}

impl<T, U: FromStreamDuration<T>> IntoStreamDuration<U> for T {
    fn into_stream_duration<C: CodecTag>(self, spec: &StreamSpec<C>) -> U {
        U::from_stream_duration(self, spec)
    }
}

impl FromStreamDuration<NFrames> for NBytes {
    fn from_stream_duration<C: CodecTag>(duration: NFrames, spec: &StreamSpec<C>) -> Self {
        todo!()
        // let n_bytes = (duration.n_frames as f64 * spec.avg_bytes_per_frame()) as u64;
        // Self { n_bytes }
    }
}

impl FromStreamDuration<NSamples> for NBytes {
    fn from_stream_duration<C: CodecTag>(duration: NSamples, spec: &StreamSpec<C>) -> Self {
        todo!()
        // let n_bytes = (duration.n_samples as f64 * spec.avg_bytes_per_sample()) as u64;
        // Self { n_bytes }
    }
}

impl FromStreamDuration<NBlocks> for NBytes {
    fn from_stream_duration<C: CodecTag>(duration: NBlocks, spec: &StreamSpec<C>) -> Self {
        let n_bytes = duration.n_blocks * spec.block_align as u64;
        Self { n_bytes }
    }
}

impl FromStreamDuration<Duration> for NBytes {
    fn from_stream_duration<C: CodecTag>(duration: Duration, spec: &StreamSpec<C>) -> Self {
        let n_bytes = (duration.as_secs_f64() * spec.byte_rate as f64) as u64;
        Self { n_bytes }
    }
}

impl FromStreamDuration<NFrames> for NBlocks {
    fn from_stream_duration<C: CodecTag>(duration: NFrames, spec: &StreamSpec<C>) -> Self {
        NBytes::from_stream_duration(duration, spec).into_stream_duration(spec)
    }
}

impl FromStreamDuration<NSamples> for NBlocks {
    fn from_stream_duration<C: CodecTag>(duration: NSamples, spec: &StreamSpec<C>) -> Self {
        NBytes::from_stream_duration(duration, spec).into_stream_duration(spec)
    }
}

impl FromStreamDuration<NBytes> for NBlocks {
    fn from_stream_duration<C: CodecTag>(duration: NBytes, spec: &StreamSpec<C>) -> Self {
        debug_assert!(
            duration.n_bytes % spec.block_align as u64 == 0,
            "n bytes not divisible by block align"
        );

        let n_blocks = duration.n_bytes / spec.block_align as u64;
        Self { n_blocks }
    }
}

impl FromStreamDuration<Duration> for NBlocks {
    fn from_stream_duration<C: CodecTag>(duration: Duration, spec: &StreamSpec<C>) -> Self {
        todo!()
        // let n_blocks = (duration.as_secs_f64() * spec.avg_block_rate()) as u64;
        // Self { n_blocks }
    }
}

impl FromStreamDuration<NBytes> for Duration {
    fn from_stream_duration<C: CodecTag>(duration: NBytes, spec: &StreamSpec<C>) -> Self {
        let seconds = duration.n_bytes as f64 / spec.byte_rate as f64;
        Duration::from_secs_f64(seconds)
    }
}

impl FromStreamDuration<NBlocks> for Duration {
    fn from_stream_duration<C: CodecTag>(duration: NBlocks, spec: &StreamSpec<C>) -> Self {
        todo!()
        // let seconds = duration.n_blocks as f64 / spec.avg_block_rate();
        // Duration::from_secs_f64(seconds)
    }
}

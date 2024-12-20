use crate::SignalSpec;
use std::{
    ops::{Add, Div, Mul, Sub},
    time::Duration,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NFrames {
    pub n_frames: u64,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct NSamples {
    pub n_samples: u64,
}

pub trait FromDuration<T> {
    fn from_duration(duration: T, spec: &SignalSpec) -> Self;
}

pub trait IntoDuration<T> {
    fn into_duration(self, spec: &SignalSpec) -> T;
}

pub trait SignalDuration:
    Sized
    + Copy
    + PartialEq
    + PartialOrd
    + IntoDuration<NFrames>
    + IntoDuration<NSamples>
    + IntoDuration<Duration>
    + FromDuration<NFrames>
    + FromDuration<NSamples>
    + FromDuration<Duration>
    + Add<Output = Self>
    + Sub<Output = Self>
{
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

        impl Sub<$unit> for $unit {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self {
                let inner = self.$inner.sub(rhs.$inner);
                Self::from(inner)
            }
        }

        impl Mul<u64> for $unit {
            type Output = Self;

            fn mul(self, rhs: u64) -> Self {
                let inner = self.$inner.mul(rhs);
                Self::from(inner)
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
    };
}

impl_ops!(NFrames, n_frames);
impl_ops!(NSamples, n_samples);

impl<T> FromDuration<T> for T {
    fn from_duration(duration: T, _spec: &SignalSpec) -> Self {
        duration
    }
}

impl<T, U: FromDuration<T>> IntoDuration<U> for T {
    fn into_duration(self, spec: &SignalSpec) -> U {
        U::from_duration(self, spec)
    }
}

impl FromDuration<NSamples> for NFrames {
    fn from_duration(duration: NSamples, spec: &SignalSpec) -> Self {
        let n_channels = spec.channels.count() as u64;
        debug_assert!(
            duration.n_samples % n_channels == 0,
            "n samples not divisible by n channels"
        );

        let n_frames = duration.n_samples / n_channels;
        Self { n_frames }
    }
}

impl FromDuration<Duration> for NFrames {
    fn from_duration(duration: Duration, spec: &SignalSpec) -> Self {
        let n_frames = (duration.as_secs_f64() * spec.sample_rate as f64) as u64;
        Self { n_frames }
    }
}

impl FromDuration<NFrames> for NSamples {
    fn from_duration(duration: NFrames, spec: &SignalSpec) -> Self {
        let n_samples = duration.n_frames * spec.channels.count() as u64;
        Self { n_samples }
    }
}

impl FromDuration<Duration> for NSamples {
    fn from_duration(duration: Duration, spec: &SignalSpec) -> Self {
        let n_frames = NFrames::from_duration(duration, spec);
        Self::from_duration(n_frames, spec)
    }
}

impl FromDuration<NFrames> for Duration {
    fn from_duration(duration: NFrames, spec: &SignalSpec) -> Self {
        let seconds = duration.n_frames as f64 / spec.sample_rate as f64;
        Self::from_secs_f64(seconds)
    }
}

impl FromDuration<NSamples> for Duration {
    fn from_duration(duration: NSamples, spec: &SignalSpec) -> Self {
        let n_frames = NFrames::from_duration(duration, spec);
        Self::from_duration(n_frames, spec)
    }
}

impl<T> SignalDuration for T where
    T: Sized
        + Copy
        + PartialEq
        + PartialOrd
        + IntoDuration<NFrames>
        + IntoDuration<NSamples>
        + IntoDuration<Duration>
        + FromDuration<NFrames>
        + FromDuration<NSamples>
        + FromDuration<Duration>
        + Add<Output = Self>
        + Sub<Output = Self>
{
}

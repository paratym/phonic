use crate::{Channels, Sample};
use phonic_core::PhonicError;
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

/// A set of parameters that describes an interleaved pcm signal
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SignalSpec {
    /// The number of samples per channel per second.
    pub frame_rate: u32,

    /// The layout of the channels in the signal.
    pub channels: Channels,

    /// The total number of sample blocks in the signal.
    pub n_frames: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub struct SignalSpecBuilder {
    pub frame_rate: Option<u32>,
    pub channels: Option<Channels>,
    pub n_frames: Option<u64>,
}

impl SignalSpec {
    pub fn builder() -> SignalSpecBuilder {
        SignalSpecBuilder::new()
    }

    pub fn sample_rate(&self) -> u64 {
        self.frame_rate as u64 * self.channels.count() as u64
    }

    pub fn n_samples(&self) -> Option<u64> {
        self.n_frames.map(|n| n * self.channels.count() as u64)
    }

    pub fn duration(&self) -> Option<Duration> {
        self.n_frames
            .map(|n| Duration::from_secs_f64(n as f64 / self.frame_rate as f64))
    }

    pub fn is_compatible(&self, other: &Self) -> bool {
        self.frame_rate == other.frame_rate && self.channels.is_compatible(&other.channels)
    }
}

impl TryFrom<SignalSpecBuilder> for SignalSpec {
    type Error = PhonicError;

    fn try_from(builder: SignalSpecBuilder) -> Result<Self, Self::Error> {
        Ok(Self {
            channels: builder.channels.ok_or(PhonicError::MissingData)?,
            frame_rate: builder.frame_rate.ok_or(PhonicError::MissingData)?,
            n_frames: builder.n_frames,
        })
    }
}

impl SignalSpecBuilder {
    pub fn new() -> Self {
        Self {
            frame_rate: None,
            channels: None,
            n_frames: None,
        }
    }

    pub fn with_frame_rate(mut self, frame_rate: u32) -> Self {
        self.frame_rate = Some(frame_rate);
        self
    }

    pub fn raw_sample_rate(&self) -> Option<u64> {
        self.frame_rate
            .zip(self.channels)
            .map(|(frame_rate, channels)| frame_rate as u64 * channels.count() as u64)
    }

    pub fn with_raw_sample_rate(mut self, sample_rate: u32) -> Result<Self, PhonicError> {
        if let Some(c) = self.channels {
            let n_channels = c.count() as u32;
            if sample_rate % n_channels != 0 {
                return Err(PhonicError::SignalMismatch);
            }

            self.frame_rate = Some(sample_rate / n_channels);
        }

        Ok(self)
    }

    pub fn with_channels(mut self, channels: impl Into<Channels>) -> Self {
        self.channels = Some(channels.into());
        self
    }

    pub fn with_n_frames(mut self, n_frames: impl Into<Option<u64>>) -> Self {
        self.n_frames = n_frames.into();
        self
    }

    pub fn n_samples(&self) -> Option<u64> {
        self.n_frames
            .zip(self.channels)
            .map(|(n_frames, channels)| n_frames * channels.count() as u64)
    }

    pub fn with_n_samples(self, n_samples: Option<u64>) -> Self {
        self.with_n_frames(
            n_samples
                .zip(self.channels)
                .map(|(n, c)| n * c.count() as u64),
        )
    }

    pub fn duration(&self) -> Option<Duration> {
        self.n_frames
            .zip(self.frame_rate)
            .map(|(n_frames, frame_rate)| {
                Duration::from_secs_f64(n_frames as f64 / frame_rate as f64)
            })
    }

    pub fn with_duration(self, duration: Duration) -> Self {
        self.with_n_frames(
            self.frame_rate
                .map(|hz| (hz as f64 * duration.as_secs_f64()) as u64),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.frame_rate.is_none() && self.channels.is_none() && self.n_frames.is_none()
    }

    pub fn merge(&mut self, other: Self) -> Result<(), PhonicError> {
        if let Some(frame_rate) = other.frame_rate {
            if self.frame_rate.get_or_insert(frame_rate) != &frame_rate {
                return Err(PhonicError::SignalMismatch);
            }
        }

        if let Some((a, b)) = self.channels.zip(other.channels) {
            self.channels = Some(a.merge(b)?);
        } else {
            self.channels = self.channels.or(other.channels)
        }

        if let Some((a, b)) = self.n_frames.zip(other.n_frames) {
            self.n_frames = Some(a.min(b));
        } else {
            self.n_frames = self.n_frames.or(other.n_frames)
        }

        Ok(())
    }

    pub fn build(self) -> Result<SignalSpec, PhonicError> {
        self.try_into()
    }
}

impl From<SignalSpec> for SignalSpecBuilder {
    fn from(spec: SignalSpec) -> Self {
        Self {
            frame_rate: Some(spec.frame_rate),
            channels: Some(spec.channels),
            n_frames: spec.n_frames,
        }
    }
}

pub trait Signal {
    type Sample: Sample;

    fn spec(&self) -> &SignalSpec;
}

pub trait SignalObserver: Signal {
    fn position(&self) -> Result<u64, PhonicError>;
}

pub trait SignalReader: Signal {
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError>;

    fn read_exact(&mut self, mut buf: &mut [Self::Sample]) -> Result<(), PhonicError> {
        if buf.len() % self.spec().channels.count() as usize != 0 {
            return Err(PhonicError::SignalMismatch);
        }

        while !buf.is_empty() {
            match self.read(&mut buf) {
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => buf = &mut buf[n..],
                Err(PhonicError::Interrupted) => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

pub trait SignalWriter: Signal {
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, PhonicError>;
    fn flush(&mut self) -> Result<(), PhonicError>;

    fn write_exact(&mut self, mut buf: &[Self::Sample]) -> Result<(), PhonicError> {
        if buf.len() % self.spec().channels.count() as usize != 0 {
            return Err(PhonicError::SignalMismatch);
        }

        while !buf.is_empty() {
            match self.write(&buf) {
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => buf = &buf[n..],
                Err(PhonicError::Interrupted) => continue,
                Err(e) => return Err(e),
            };
        }

        Ok(())
    }

    fn copy_n_buffered<R>(
        &mut self,
        reader: &mut R,
        n: u64,
        buf: &mut [Self::Sample],
    ) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        let spec = self.spec();
        if !spec.is_compatible(reader.spec()) || n % spec.channels.count() as u64 != 0 {
            return Err(PhonicError::SignalMismatch);
        }

        let mut n_read = 0;
        while n_read < n {
            let buf_len = buf.len().min((n - n_read) as usize);
            let n = match reader.read(&mut buf[..buf_len]) {
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => n,
                Err(PhonicError::Interrupted) => continue,
                Err(e) => return Err(e),
            };

            self.write_exact(&buf[..n])?;
            n_read += n as u64;
        }

        Ok(())
    }

    fn copy_n<R>(&mut self, reader: &mut R, n: u64) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        let mut buf = [Self::Sample::ORIGIN; 8096];
        self.copy_n_buffered(reader, n, &mut buf)
    }

    fn copy_all_buffered<R>(
        &mut self,
        reader: &mut R,
        buf: &mut [Self::Sample],
    ) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        let n = u64::MAX - (u64::MAX % self.spec().channels.count() as u64);
        match self.copy_n_buffered(reader, n, buf) {
            Ok(_) | Err(PhonicError::OutOfBounds) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn copy_all<R>(&mut self, reader: &mut R) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        let n = u64::MAX - (u64::MAX % self.spec().channels.count() as u64);
        self.copy_n(reader, n)
    }
}

pub trait SignalSeeker: Signal {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError>;

    fn set_position(&mut self, position: u64) -> Result<(), PhonicError>
    where
        Self: SignalObserver,
    {
        self.seek(self.position()? as i64 - position as i64)
    }
}

impl<T, S> Signal for T
where
    S: Sample,
    T: Deref,
    T::Target: Signal<Sample = S>,
{
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        self.deref().spec()
    }
}

impl<T, S> SignalObserver for T
where
    S: Sample,
    T: Deref,
    T::Target: SignalObserver<Sample = S>,
{
    fn position(&self) -> Result<u64, PhonicError> {
        self.deref().position()
    }
}

impl<S, T> SignalReader for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalReader<Sample = S>,
{
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, PhonicError> {
        self.deref_mut().read(buffer)
    }
}

impl<S, T> SignalWriter for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalWriter<Sample = S>,
{
    fn write(&mut self, buffer: &[S]) -> Result<usize, PhonicError> {
        self.deref_mut().write(buffer)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.deref_mut().flush()
    }
}

impl<S, T> SignalSeeker for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalSeeker<Sample = S>,
{
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        self.deref_mut().seek(offset)
    }
}

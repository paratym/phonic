use crate::{Channels, Sample};
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};
use syphon_core::SyphonError;

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
}

impl TryFrom<SignalSpecBuilder> for SignalSpec {
    type Error = SyphonError;

    fn try_from(builder: SignalSpecBuilder) -> Result<Self, Self::Error> {
        Ok(Self {
            channels: builder.channels.ok_or(SyphonError::MissingData)?,
            frame_rate: builder.frame_rate.ok_or(SyphonError::MissingData)?,
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

    pub fn sample_rate(&self) -> Option<u64> {
        self.frame_rate
            .zip(self.channels)
            .map(|(frame_rate, channels)| frame_rate as u64 * channels.count() as u64)
    }

    pub fn n_samples(&self) -> Option<u64> {
        self.n_frames
            .zip(self.channels)
            .map(|(n_frames, channels)| n_frames * channels.count() as u64)
    }

    pub fn duration(&self) -> Option<Duration> {
        self.n_frames
            .zip(self.frame_rate)
            .map(|(n_frames, frame_rate)| {
                Duration::from_secs_f64(n_frames as f64 / frame_rate as f64)
            })
    }

    pub fn with_frame_rate(mut self, frame_rate: u32) -> Self {
        self.frame_rate = Some(frame_rate);
        self
    }

    pub fn with_channels(mut self, channels: impl Into<Channels>) -> Self {
        self.channels = Some(channels.into());
        self
    }

    pub fn with_n_frames(mut self, n_frames: impl Into<Option<u64>>) -> Self {
        self.n_frames = n_frames.into();
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.n_frames = self
            .frame_rate
            .map(|hz| (hz as f64 * duration.as_secs_f64()) as u64);

        self
    }

    pub fn is_empty(&self) -> bool {
        self.frame_rate.is_none() && self.channels.is_none() && self.n_frames.is_none()
    }

    pub fn merge(&mut self, other: Self) -> Result<(), SyphonError> {
        if let Some(frame_rate) = other.frame_rate {
            if self.frame_rate.get_or_insert(frame_rate) != &frame_rate {
                return Err(SyphonError::SignalMismatch);
            }
        }

        if let Some((self_ch, other_ch)) = self.channels.zip(other.channels) {
            if self_ch.count() != other_ch.count() {
                return Err(SyphonError::SignalMismatch);
            }

            match (self_ch, other_ch) {
                (Channels::Layout(s), Channels::Layout(o)) if s != o => {
                    return Err(SyphonError::SignalMismatch);
                }
                (Channels::Count(_), Channels::Layout(_)) => {
                    self.channels = Some(other_ch);
                }
                _ => {}
            }
        } else {
            self.channels = self.channels.or(other.channels);
        }

        if let Some(n_frames) = other.n_frames {
            if self.n_frames.get_or_insert(n_frames) != &n_frames {
                return Err(SyphonError::SignalMismatch);
            }
        }

        Ok(())
    }

    pub fn build(self) -> Result<SignalSpec, SyphonError> {
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

pub trait SignalReader: Signal {
    fn read(&mut self, buffer: &mut [Self::Sample]) -> Result<usize, SyphonError>;

    fn read_exact(&mut self, mut buffer: &mut [Self::Sample]) -> Result<(), SyphonError> {
        while !buffer.is_empty() {
            match self.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => buffer = &mut buffer[n..],
                Err(SyphonError::Interrupted) => continue,
                Err(e) => return Err(e),
            };
        }

        if buffer.len() > 0 {
            return Err(SyphonError::EndOfStream);
        }

        Ok(())
    }

    fn pipe_buffered<W>(
        &mut self,
        writer: &mut W,
        buffer: &mut [Self::Sample],
    ) -> Result<u64, SyphonError>
    where
        Self: Sized,
        W: SignalWriter<Sample = Self::Sample>,
    {
        let spec = self.spec();
        if spec != writer.spec() {
            return Err(SyphonError::SignalMismatch);
        }

        let mut n_read = 0;
        loop {
            let n = match self.read(buffer) {
                Ok(0) | Err(SyphonError::EndOfStream) => return Ok(n_read),
                Ok(n) => n,
                Err(SyphonError::Interrupted) | Err(SyphonError::NotReady) => continue,
                Err(e) => return Err(e),
            };

            writer.write_exact(&buffer[..n])?;
            n_read += n as u64;
        }
    }

    fn pipe<W>(&mut self, writer: &mut W) -> Result<u64, SyphonError>
    where
        Self: Sized,
        W: SignalWriter<Sample = Self::Sample>,
    {
        let mut buffer = [Self::Sample::ORIGIN; 2048];
        self.pipe_buffered(writer, &mut buffer)
    }
}

pub trait SignalWriter: Signal {
    fn write(&mut self, buffer: &[Self::Sample]) -> Result<usize, SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;

    fn write_exact(&mut self, mut buffer: &[Self::Sample]) -> Result<(), SyphonError> {
        while !buffer.is_empty() {
            match self.write(&buffer) {
                Ok(0) => break,
                Ok(n) => buffer = &buffer[n..],
                Err(SyphonError::Interrupted) => continue,
                Err(e) => return Err(e),
            };
        }

        if buffer.len() > 0 {
            return Err(SyphonError::EndOfStream);
        }

        Ok(())
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

impl<S, T> SignalReader for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalReader<Sample = S>,
{
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        self.deref_mut().read(buffer)
    }
}

impl<S, T> SignalWriter for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalWriter<Sample = S>,
{
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        self.deref_mut().write(buffer)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.deref_mut().flush()
    }
}

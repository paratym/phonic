use crate::{Channels, PhonicError, PhonicResult};
use std::{f64, time::Duration};

/// A set of parameters that describes an interleaved pcm signal
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SignalSpec {
    /// The number of samples per channel per second.
    pub sample_rate: u32,

    /// The count or layout of the channels in the signal.
    pub channels: Channels,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SignalSpecBuilder {
    pub sample_rate: Option<u32>,
    pub channels: Option<Channels>,
}

impl SignalSpec {
    pub fn new(sample_rate: u32, channels: impl Into<Channels>) -> Self {
        Self {
            sample_rate,
            channels: channels.into(),
        }
    }

    pub fn builder() -> SignalSpecBuilder {
        SignalSpecBuilder::new()
    }

    pub fn into_builder(self) -> SignalSpecBuilder {
        self.into()
    }

    pub fn sample_rate_interleaved(&self) -> u32 {
        self.sample_rate * self.channels.count()
    }

    pub fn sample_rate_duration(&self) -> Duration {
        let seconds = 1.0 / self.sample_rate as f64;
        Duration::from_secs_f64(seconds)
    }

    pub fn merge(&mut self, other: &Self) -> PhonicResult<()> {
        if self.sample_rate != other.sample_rate {
            return Err(PhonicError::param_mismatch());
        }

        self.channels.merge(&other.channels)?;
        Ok(())
    }

    pub fn merged(mut self, other: &Self) -> PhonicResult<Self> {
        self.merge(other)?;
        Ok(self)
    }
}

impl SignalSpecBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sample_rate_interleaved(&self) -> Option<u32> {
        self.sample_rate
            .zip(self.channels)
            .map(|(r, c)| r * c.count())
    }

    pub fn sample_rate_duration(&self) -> Option<Duration> {
        self.sample_rate
            .map(|r| 1.0 / r as f64)
            .map(Duration::from_secs_f64)
    }

    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    pub fn with_sample_rate_interleaved(self, sample_rate: u32) -> Self {
        match self.channels {
            Some(c) => self.with_sample_rate(sample_rate / c.count()),
            None => self,
        }
    }

    pub fn with_sample_rate_duration(self, interval: Duration) -> Self {
        self.with_sample_rate((1.0 / interval.as_secs_f64()) as u32)
    }

    pub fn with_channels(mut self, channels: impl Into<Channels>) -> Self {
        self.channels = Some(channels.into());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.sample_rate.is_none() && self.channels.is_none()
    }

    pub fn is_full(&self) -> bool {
        self.sample_rate.is_some() && self.channels.is_some()
    }

    pub fn merge(&mut self, other: &Self) -> PhonicResult<()> {
        if let Some(sample_rate) = other.sample_rate {
            if self.sample_rate.get_or_insert(sample_rate) != &sample_rate {
                return Err(PhonicError::param_mismatch());
            }
        }

        self.channels = match (self.channels, other.channels) {
            (Some(a), Some(ref b)) => a.merged(b)?.into(),
            (Some(c), _) | (_, Some(c)) => c.into(),
            _ => None,
        };

        Ok(())
    }

    pub fn merged(mut self, other: &Self) -> PhonicResult<Self> {
        self.merge(other)?;
        Ok(self)
    }

    pub fn build(self) -> PhonicResult<SignalSpec> {
        self.try_into()
    }
}

impl TryFrom<SignalSpecBuilder> for SignalSpec {
    type Error = PhonicError;

    fn try_from(builder: SignalSpecBuilder) -> Result<Self, Self::Error> {
        Ok(Self {
            channels: builder.channels.ok_or(PhonicError::missing_data())?,
            sample_rate: builder.sample_rate.ok_or(PhonicError::missing_data())?,
        })
    }
}

impl From<SignalSpec> for SignalSpecBuilder {
    fn from(spec: SignalSpec) -> Self {
        Self {
            sample_rate: Some(spec.sample_rate),
            channels: Some(spec.channels),
        }
    }
}

use crate::{PhonicError, PhonicResult};

/// A set of parameters that describes an interleaved pcm signal
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SignalSpec {
    /// The number of samples per channel per second.
    pub sample_rate: usize,

    /// The number of interleaved channels in this signal.
    pub n_channels: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SignalSpecBuilder {
    pub sample_rate: Option<usize>,
    pub n_channels: Option<usize>,
}

impl SignalSpec {
    pub fn new(n_channels: usize, sample_rate: usize) -> Self {
        Self {
            sample_rate,
            n_channels,
        }
    }

    pub fn mono(sample_rate: usize) -> Self {
        Self::new(1, sample_rate)
    }

    pub fn stereo(sample_rate: usize) -> Self {
        Self::new(2, sample_rate)
    }

    pub fn builder() -> SignalSpecBuilder {
        SignalSpecBuilder::new()
    }

    pub fn into_builder(self) -> SignalSpecBuilder {
        self.into()
    }
}

impl SignalSpecBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sample_rate(mut self, sample_rate: impl Into<Option<usize>>) -> Self {
        self.sample_rate = sample_rate.into();
        self
    }

    pub fn with_n_channels(mut self, channels: impl Into<Option<usize>>) -> Self {
        self.n_channels = channels.into();
        self
    }

    pub fn is_empty(&self) -> bool {
        self.sample_rate.is_none() && self.n_channels.is_none()
    }

    pub fn is_full(&self) -> bool {
        self.sample_rate.is_some() && self.n_channels.is_some()
    }

    pub fn merge(&mut self, other: &Self) -> PhonicResult<()> {
        if let Some(sample_rate) = other.sample_rate {
            if self.sample_rate.get_or_insert(sample_rate) != &sample_rate {
                return Err(PhonicError::param_mismatch());
            }
        }

        if let Some(n_channels) = other.n_channels {
            if self.n_channels.get_or_insert(n_channels) != &n_channels {
                return Err(PhonicError::param_mismatch());
            }
        }

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
            n_channels: builder.n_channels.ok_or(PhonicError::missing_data())?,
            sample_rate: builder.sample_rate.ok_or(PhonicError::missing_data())?,
        })
    }
}

impl From<SignalSpec> for SignalSpecBuilder {
    fn from(spec: SignalSpec) -> Self {
        Self {
            sample_rate: Some(spec.sample_rate),
            n_channels: Some(spec.n_channels),
        }
    }
}

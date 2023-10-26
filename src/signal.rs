use crate::{Sample, SampleFormat, SyphonError};

/// A set of parameters that describes a pcm signal
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SignalSpec {
    pub sample_format: SampleFormat,

    /// The sample rate in hertz
    pub sample_rate: u32,

    /// The number of channels in the signal.
    pub n_channels: u8,

    /// The minimum number of frames that can be read or written at once.
    /// This does not need to be enforced by the consumer, but can be useful for sizing buffers.
    pub block_size: usize,

    /// The total number of blocks in the signal.
    pub n_blocks: Option<u64>,
}

#[derive(Default, Clone, Copy)]
pub struct SignalSpecBuilder {
    pub sample_format: Option<SampleFormat>,
    pub sample_rate: Option<u32>,
    pub n_channels: Option<u8>,
    pub block_size: Option<usize>,
    pub n_blocks: Option<u64>,
}

impl SignalSpec {
    pub fn builder() -> SignalSpecBuilder {
        SignalSpecBuilder::new()
    }

    pub fn samples_per_block(&self) -> usize {
        self.block_size * self.n_channels as usize
    }

    pub fn n_frames(&self) -> Option<u64> {
        self.n_blocks.map(|n| n * self.block_size as u64)
    }

    pub fn n_samples(&self) -> Option<u64> {
        self.n_frames().map(|n| n * self.n_channels as u64)
    }
}

impl TryFrom<SignalSpecBuilder> for SignalSpec {
    type Error = SyphonError;

    fn try_from(builder: SignalSpecBuilder) -> Result<Self, Self::Error> {
        Ok(Self {
            sample_format: builder.sample_format.ok_or(SyphonError::InvalidData)?,
            n_channels: builder.n_channels.ok_or(SyphonError::InvalidData)?,
            sample_rate: builder.sample_rate.ok_or(SyphonError::InvalidData)?,
            block_size: builder.block_size.unwrap_or(1),
            n_blocks: builder.n_blocks,
        })
    }
}

impl SignalSpecBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn samples_per_block(&self) -> Option<usize> {
        self.block_size.and_then(|block_size| {
            self.n_channels
                .map(|n_channels| block_size * n_channels as usize)
        })
    }

    pub fn n_frames(&self) -> Option<u64> {
        self.n_blocks.and_then(|n_blocks| {
            self.block_size
                .map(|block_size| n_blocks * block_size as u64)
        })
    }

    pub fn n_samples(&self) -> Option<u64> {
        self.n_frames().and_then(|n_frames| {
            self.n_channels
                .map(|n_channels| n_frames * n_channels as u64)
        })
    }

    pub fn is_empty(&self) -> bool {
        self.sample_format.is_none()
            && self.n_channels.is_none()
            && self.sample_rate.is_none()
            && self.block_size.is_none()
            && self.n_blocks.is_none()
    }

    pub fn sample_format(mut self, sample_format: SampleFormat) -> Self {
        self.sample_format = Some(sample_format);
        self
    }

    pub fn sample_type<S: Sample>(mut self) -> Self {
        self.sample_format = Some(S::FORMAT);
        self
    }

    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    pub fn n_channels(mut self, n_channels: u8) -> Self {
        self.n_channels = Some(n_channels);
        self
    }

    pub fn block_size(mut self, block_size: usize) -> Self {
        self.block_size = Some(block_size);
        self
    }

    pub fn n_blocks(mut self, n_blocks: u64) -> Self {
        self.n_blocks = Some(n_blocks);
        self
    }

    pub fn build(self) -> Result<SignalSpec, SyphonError> {
        self.try_into()
    }
}

impl From<SignalSpec> for SignalSpecBuilder {
    fn from(spec: SignalSpec) -> Self {
        Self {
            sample_format: Some(spec.sample_format),
            sample_rate: Some(spec.sample_rate),
            n_channels: Some(spec.n_channels),
            block_size: Some(spec.block_size),
            n_blocks: spec.n_blocks,
        }
    }
}

pub trait Signal {
    fn spec(&self) -> &SignalSpec;
}

pub trait SignalReader<S: Sample>: Signal {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;

    fn read_exact(&mut self, buffer: &mut [S]) -> Result<(), SyphonError> {
        let buf_len = buffer.len();
        if buf_len % self.spec().samples_per_block() != 0 {
            return Err(SyphonError::SignalMismatch);
        }

        let mut n_read: usize = 0;
        while n_read < buf_len {
            match self.read(&mut buffer[n_read..]) {
                Ok(0) => break,
                Ok(n) => n_read += n,
                Err(SyphonError::Interrupted) => continue,
                Err(e) => return Err(e),
            }
        }

        if n_read != buf_len {
            return Err(SyphonError::EndOfStream);
        }

        Ok(())
    }
}

pub trait SignalWriter<S: Sample>: Signal {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;

    fn write_exact(&mut self, buffer: &[S]) -> Result<(), SyphonError> {
        let buf_len = buffer.len();
        if buf_len % self.spec().samples_per_block() != 0 {
            return Err(SyphonError::SignalMismatch);
        }

        let mut n_written: usize = 0;
        while n_written < buf_len {
            match self.write(&buffer[n_written..]) {
                Ok(0) => break,
                Ok(n) => n_written += n,
                Err(SyphonError::Interrupted) => continue,
                Err(e) => return Err(e),
            }
        }

        if n_written != buf_len {
            return Err(SyphonError::EndOfStream);
        }

        Ok(())
    }
}

impl<S: Sample> Signal for Box<dyn SignalReader<S>> {
    fn spec(&self) -> &SignalSpec {
        self.as_ref().spec()
    }
}

impl<S: Sample> SignalReader<S> for Box<dyn SignalReader<S>> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        self.as_mut().read(buffer)
    }
}

impl<S: Sample> Signal for Box<dyn SignalWriter<S>> {
    fn spec(&self) -> &SignalSpec {
        self.as_ref().spec()
    }
}

impl<S: Sample> SignalWriter<S> for Box<dyn SignalWriter<S>> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        self.as_mut().write(buffer)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.as_mut().flush()
    }
}

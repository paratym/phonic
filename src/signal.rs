use crate::{Sample, SampleFormat, SyphonError};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct StreamSpec {
    pub sample_format: SampleFormat,
    pub n_channels: u8,
    pub sample_rate: u32,
    pub block_size: usize,
    pub n_frames: Option<u64>,
}

#[derive(Default, Clone, Copy)]
pub struct StreamSpecBuilder {
    pub sample_format: Option<SampleFormat>,
    pub n_channels: Option<u8>,
    pub sample_rate: Option<u32>,
    pub block_size: Option<usize>,
    pub n_frames: Option<u64>,
}

impl StreamSpec {
    pub fn builder() -> StreamSpecBuilder {
        StreamSpecBuilder::new()
    }

    pub fn frames_per_block(&self) -> usize {
        self.block_size / self.n_channels as usize
    }

    pub fn n_blocks(&self) -> Option<u64> {
        self.n_frames.map(|n| n / self.block_size as u64)
    }
}

impl TryFrom<StreamSpecBuilder> for StreamSpec {
    type Error = SyphonError;

    fn try_from(builder: StreamSpecBuilder) -> Result<Self, Self::Error> {
        if builder
            .block_size
            .zip(builder.n_channels)
            .map_or(false, |(b, n)| b % n as usize != 0)
        {
            return Err(SyphonError::InvalidData);
        }

        if builder
            .n_frames
            .zip(builder.block_size)
            .map_or(false, |(n, b)| n % b as u64 != 0)
        {
            return Err(SyphonError::InvalidData);
        }

        Ok(Self {
            sample_format: builder.sample_format.ok_or(SyphonError::InvalidData)?,
            n_channels: builder.n_channels.ok_or(SyphonError::InvalidData)?,
            sample_rate: builder.sample_rate.ok_or(SyphonError::InvalidData)?,
            block_size: builder.block_size.ok_or(SyphonError::InvalidData)?,
            n_frames: builder.n_frames,
        })
    }
}

impl StreamSpecBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn frames_per_block(&self) -> Option<usize> {
        self.block_size
            .zip(self.n_channels)
            .map(|(b, n)| b / n as usize)
    }

    pub fn n_blocks(&self) -> Option<u64> {
        self.n_frames
            .zip(self.block_size)
            .map(|(n, b)| n / b as u64)
    }

    pub fn is_empty(&self) -> bool {
        self.sample_format.is_none()
            && self.n_channels.is_none()
            && self.sample_rate.is_none()
            && self.block_size.is_none()
            && self.n_frames.is_none()
    }

    pub fn sample_type(mut self, sample_format: SampleFormat) -> Self {
        self.sample_format = Some(sample_format);
        self
    }

    pub fn channels(mut self, n_channels: u8) -> Self {
        self.n_channels = Some(n_channels);
        self
    }

    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    pub fn block_size(mut self, block_size: usize) -> Self {
        self.block_size = Some(block_size);
        self
    }

    pub fn n_frames(mut self, n_frames: u64) -> Self {
        self.n_frames = Some(n_frames);
        self
    }

    pub fn build(self) -> Result<StreamSpec, SyphonError> {
        self.try_into()
    }
}

impl From<StreamSpec> for StreamSpecBuilder {
    fn from(spec: StreamSpec) -> Self {
        Self {
            sample_format: Some(spec.sample_format),
            n_channels: Some(spec.n_channels),
            sample_rate: Some(spec.sample_rate),
            block_size: Some(spec.block_size),
            n_frames: spec.n_frames,
        }
    }
}

pub trait SampleStream<S: Sample> {
    fn spec(&self) -> &StreamSpec;
}

pub trait SampleReader<S: Sample>: SampleStream<S> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;

    fn read_exact(&mut self, mut buffer: &mut [S]) -> Result<(), SyphonError> {
        let block_size = self.spec().block_size;
        if buffer.len() % block_size != 0 {
            return Err(SyphonError::StreamMismatch);
        }

        let mut n_read: usize = 0;
        while !buffer.is_empty() {
            n_read += self.read(buffer)?;
            if n_read == 0 {
                return Err(SyphonError::EndOfStream);
            }

            buffer = &mut buffer[n_read..];
        }

        Ok(())
    }
}

pub trait SampleWriter<S: Sample>: SampleStream<S> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;

    fn write_exact(&mut self, mut buffer: &[S]) -> Result<(), SyphonError> {
        let block_size = self.spec().block_size;
        if buffer.len() % block_size != 0 {
            return Err(SyphonError::StreamMismatch);
        }

        let mut n_written: usize = 0;
        while !buffer.is_empty() {
            n_written += self.write(buffer)?;
            if n_written == 0 {
                return Err(SyphonError::EndOfStream);
            }

            buffer = &buffer[n_written..];
        }

        Ok(())
    }
}

impl<S: Sample> SampleStream<S> for Box<dyn SampleReader<S>> {
    fn spec(&self) -> &StreamSpec {
        self.as_ref().spec()
    }
}

impl<S: Sample> SampleReader<S> for Box<dyn SampleReader<S>> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        self.as_mut().read(buffer)
    }
}

impl<S: Sample> SampleStream<S> for Box<dyn SampleWriter<S>> {
    fn spec(&self) -> &StreamSpec {
        self.as_ref().spec()
    }
}

impl<S: Sample> SampleWriter<S> for Box<dyn SampleWriter<S>> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        self.as_mut().write(buffer)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.as_mut().flush()
    }
}

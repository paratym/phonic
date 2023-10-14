use crate::core::{Sample, SampleFormat, SyphonError};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SignalSpec {
    pub n_channels: u16,
    pub sample_rate: u32,
    pub block_size: usize,
    pub sample_format: SampleFormat,
    pub bytes_per_sample: u16,
}

#[derive(Default, Clone, Copy)]
pub struct SignalSpecBuilder {
    pub n_channels: Option<u16>,
    pub sample_rate: Option<u32>,
    pub block_size: Option<usize>,
    pub sample_format: Option<SampleFormat>,
    pub bytes_per_sample: Option<u16>,
}

impl SignalSpecBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn n_channels(mut self, n_channels: u16) -> Self {
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

    pub fn sample_format(mut self, sample_format: SampleFormat) -> Self {
        self.sample_format = Some(sample_format);
        self
    }

    pub fn bytes_per_sample(mut self, bytes_per_sample: u16) -> Self {
        self.bytes_per_sample = Some(bytes_per_sample);
        self
    }

    pub fn try_build(self) -> Result<SignalSpec, SyphonError> {
        Ok(SignalSpec {
            n_channels: self.n_channels.ok_or(SyphonError::MalformedData)?,
            sample_rate: self.sample_rate.ok_or(SyphonError::MalformedData)?,
            block_size: self.block_size.ok_or(SyphonError::MalformedData)?,
            sample_format: self.sample_format.ok_or(SyphonError::MalformedData)?,
            bytes_per_sample: self.bytes_per_sample.ok_or(SyphonError::MalformedData)?,
        })
    }
}

pub trait SampleReader<S: Sample> {
    fn signal_spec(&self) -> SignalSpec;
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;

    fn read_exact(&mut self, mut buffer: &mut [S]) -> Result<(), SyphonError> {
        let block_size = self.signal_spec().block_size;
        if buffer.len() % block_size != 0 {
            return Err(SyphonError::SignalMismatch);
        }

        while !buffer.is_empty() {
            let n_read = self.read(buffer)?;
            if n_read == 0 {
                return Err(SyphonError::Empty);
            } else if n_read % block_size != 0 {
                return Err(SyphonError::MalformedData);
            }

            buffer = &mut buffer[n_read..];
        }

        Ok(())
    }
}

pub enum SampleReaderRef {
    I8(Box<dyn SampleReader<i8>>),
    I16(Box<dyn SampleReader<i16>>),
    I32(Box<dyn SampleReader<i32>>),
    I64(Box<dyn SampleReader<i64>>),

    U8(Box<dyn SampleReader<u8>>),
    U16(Box<dyn SampleReader<u16>>),
    U32(Box<dyn SampleReader<u32>>),
    U64(Box<dyn SampleReader<u64>>),

    F32(Box<dyn SampleReader<f32>>),
    F64(Box<dyn SampleReader<f64>>),
}

pub trait SampleWriter<S: Sample> {
    fn signal_spec(&self) -> SignalSpec;
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError>;

    fn write_exact(&mut self, mut buffer: &[S]) -> Result<(), SyphonError> {
        let block_size = self.signal_spec().block_size;
        if buffer.len() % block_size != 0 {
            return Err(SyphonError::SignalMismatch);
        }

        while !buffer.is_empty() {
            let n_written = self.write(buffer)?;
            if n_written == 0 {
                return Err(SyphonError::Empty);
            } else if n_written % block_size != 0 {
                return Err(SyphonError::MalformedData);
            }

            buffer = &buffer[n_written..];
        }

        Ok(())
    }
}

pub fn pipe_buffered<S: Sample>(
    reader: &mut dyn SampleReader<S>,
    writer: &mut dyn SampleWriter<S>,
    buffer: &mut [S],
) -> Result<(), SyphonError> {
    let spec = reader.signal_spec();
    if writer.signal_spec() != spec {
        return Err(SyphonError::SignalMismatch);
    } else if buffer.len() % spec.block_size != 0 {
        return Err(SyphonError::SignalMismatch);
    }

    loop {
        match reader.read(buffer) {
            Ok(0) => return Ok(()),
            Ok(n) if n % spec.block_size == 0 => {}
            Ok(_) => return Err(SyphonError::MalformedData),
            Err(SyphonError::Empty) => return Ok(()),
            Err(e) => return Err(e),
        }

        writer.write_exact(buffer)?;
    }
}

pub fn pipe<S: Sample>(
    reader: &mut dyn SampleReader<S>,
    writer: &mut dyn SampleWriter<S>,
) -> Result<(), SyphonError> {
    if reader.signal_spec() != writer.signal_spec() {
        return Err(SyphonError::SignalMismatch);
    }

    let mut buffer = vec![S::MID; reader.signal_spec().block_size as usize];
    pipe_buffered(reader, writer, buffer.as_mut_slice())
}

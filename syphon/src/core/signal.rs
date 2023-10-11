use super::{Sample, SampleFormat, SyphonError};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SignalSpec {
    pub n_channels: u16,
    pub bits_per_sample: u16,
    pub sample_format: SampleFormat,
    pub sample_rate: u32,
    pub block_size: usize,
}

pub trait SampleReader<S: Sample> {
    fn signal_spec(&self) -> SignalSpec;
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;

    fn read_exact(&mut self, mut buffer: &mut [S]) -> Result<(), SyphonError> {
        let block_size = self.signal_spec().block_size as usize;
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

pub trait SampleWriter<S: Sample> {
    fn signal_spec(&self) -> SignalSpec;
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError>;

    fn write_exact(&mut self, mut buffer: &[S]) -> Result<(), SyphonError> {
        let block_size = self.signal_spec().block_size as usize;
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
            Ok(n) if n % spec.block_size == 0 => {},
            Ok(_) => return Err(SyphonError::MalformedData),
            Err(SyphonError::Empty) => return Ok(()),
            Err(e) => return Err(e),
        }

        writer.write_exact(buffer)?;
    }
}

pub fn pipe<S: Sample>(reader: &mut dyn SampleReader<S>, writer: &mut dyn SampleWriter<S>) -> Result<(), SyphonError> {
    if reader.signal_spec() != writer.signal_spec() {
        return Err(SyphonError::SignalMismatch);
    }

    let mut buffer = vec![S::MID; reader.signal_spec().block_size as usize];
    pipe_buffered(reader, writer, buffer.as_mut_slice())
}
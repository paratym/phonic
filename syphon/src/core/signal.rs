use super::{SampleFormat, Sample, SyphonError};

#[derive(Copy, Clone)]
pub struct SignalSpec {
    pub n_channels: u16,
    pub bits_per_sample: u16,
    pub sample_format: SampleFormat,
    pub sample_rate: u32,
    pub block_size: u32,
}

pub trait Source<S: Sample> {
    fn signal_spec(&self) -> SignalSpec;
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;
}

impl<S: Sample> dyn Source<S> {
    pub fn drain(&mut self, mut sink: impl Sink<S>) -> Result<(), SyphonError> {
        let mut buffer = vec![S::MID; self.signal_spec().block_size as usize];
        loop {
            let n_read = self.read(&mut buffer)?;
            sink.write(&buffer[..n_read])?;
        }
    }
}

pub trait Sink<S: Sample> {
    fn signal_spec(&self) -> SignalSpec;
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError>;
}
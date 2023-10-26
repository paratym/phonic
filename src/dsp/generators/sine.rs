use crate::{SampleFormat, SignalReader, Signal, SignalSpec, SyphonError};

pub struct SineGenerator {
    spec: SignalSpec,
    frequency: f32,
    n_read: usize,
}

impl SineGenerator {
    pub fn new(frequency: f32, sample_rate: u32) -> Self {
        let spec = SignalSpec {
            sample_format: SampleFormat::I32,
            sample_rate,
            n_channels: 1,
            block_size: 1,
            n_blocks: None,
        };

        Self {
            spec,
            frequency,
            n_read: 0,
        }
    }
}

impl Signal for SineGenerator {
    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl SignalReader<f32> for SineGenerator {
    fn read(&mut self, buffer: &mut [f32]) -> Result<usize, SyphonError> {
        for s in buffer.iter_mut() {
            let t = self.n_read as f32 / self.spec.sample_rate as f32;
            *s = (t * self.frequency * 2.0 * std::f32::consts::PI).sin();
        }

        self.n_read += buffer.len();
        Ok(buffer.len())
    }
}

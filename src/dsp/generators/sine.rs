use crate::{FromSample, IntoSample, Sample, Signal, SignalReader, SignalSpec, SyphonError};
use std::f32::consts::PI;

pub struct SineGenerator {
    spec: SignalSpec,
    frequency: f32,
    i: u64,
}

impl SineGenerator {
    pub fn new(spec: SignalSpec, frequency: f32) -> Self {
        Self {
            spec,
            frequency,
            i: 0,
        }
    }
}

impl Signal for SineGenerator {
    type Sample = f32;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl SignalReader for SineGenerator {
    fn read(&mut self, buffer: &mut [Self::Sample]) -> Result<usize, SyphonError> {
        let buf_len = self
            .spec
            .n_samples()
            .map(|n| ((n - self.i) as usize).min(buffer.len()))
            .unwrap_or(buffer.len());

        let n_channels = self.spec.channels.count() as usize;
        let mut frames = buffer[..buf_len].chunks_exact_mut(n_channels);
        let n_frames = frames.len();

        for frame in &mut frames {
            let t = self.i as f32 / self.spec.frame_rate as f32;
            frame.fill((t * self.frequency * 2.0 * PI).sin());

            self.i += 1;
        }

        let n_samples = n_frames * n_channels;
        Ok(n_samples)
    }
}

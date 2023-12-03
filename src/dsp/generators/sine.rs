use crate::{FromSample, IntoSample, Sample, Signal, SignalReader, SignalSpec, SyphonError};
use std::f32::consts::PI;

pub struct Sine<S: Sample> {
    spec: SignalSpec<S>,
    frequency: f32,
    i: u64,
}

impl<S: Sample> Sine<S> {
    pub fn new(spec: SignalSpec<S>, frequency: f32) -> Self {
        Self {
            spec,
            frequency,
            i: 0,
        }
    }
}

impl<S: Sample> Signal<S> for Sine<S> {
    fn spec(&self) -> &SignalSpec<S> {
        &self.spec
    }
}

impl<S: Sample + FromSample<f32>> SignalReader<S> for Sine<S> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        let mut buf_len = buffer.len();
        if let Some(n) = self.spec.n_samples() {
            let sample_index = self.i * self.spec.channels.count() as u64;
            buf_len = buf_len.min((n - sample_index) as usize);
        }

        buf_len -= buf_len % self.spec.samples_per_block();

        let mut frames = buffer[..buf_len].chunks_exact_mut(self.spec.channels.count() as usize);
        let n_frames = frames.len();

        for frame in &mut frames {
            let t = self.i as f32 / self.spec.frame_rate as f32;
            frame.fill((t * self.frequency * 2.0 * PI).sin().into_sample());

            self.i += 1;
        }

        Ok(n_frames / self.spec.block_size)
    }
}

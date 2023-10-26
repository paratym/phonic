use crate::{FromSample, IntoSample, Sample, Signal, SignalReader, SignalSpec, SyphonError};

pub struct Sine {
    spec: SignalSpec,
    frequency: f32,
    i: u64,
}

impl Sine {
    pub fn new(spec: SignalSpec, frequency: f32) -> Self {
        Self {
            spec,
            frequency,
            i: 0,
        }
    }
}

impl Signal for Sine {
    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<S: Sample + FromSample<f32>> SignalReader<S> for Sine {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        if S::FORMAT != self.spec.sample_format {
            return Err(SyphonError::SignalMismatch);
        } else if self.spec.n_samples().is_some_and(|n| self.i >= n) {
            return Ok(0);
        }

        let mut buf_len = buffer.len();
        buf_len = self
            .spec
            .n_samples()
            .map_or(buf_len, |n| buf_len.min((n - self.i) as usize));

        buf_len -= buf_len % self.spec.samples_per_block();

        let frames = &mut buffer[..buf_len]
            .chunks_exact_mut(self.spec.n_channels as usize)
            .into_iter();

        for frame in frames {
            let t = self.i as f32 / self.spec.sample_rate as f32;
            frame.fill(
                (t * self.frequency * 2.0 * std::f32::consts::PI)
                    .sin()
                    .into_sample(),
            );
        }

        self.i += buf_len as u64;
        Ok(buf_len)
    }
}

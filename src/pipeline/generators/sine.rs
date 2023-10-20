use std::io::SeekFrom;

use crate::{
    io::{SampleReader, StreamSpec},
    Sample, SampleFormat, SyphonError,
};

pub struct SineGenerator {
    stream_spec: StreamSpec,
    frequency: f32,
    n_read: usize,
}

impl SineGenerator {
    pub fn new(frequency: f32) -> Self {
        let stream_spec = StreamSpec {
            sample_format: SampleFormat::I32,
            sample_rate: 44100,
            n_channels: 1,
            block_size: 1,
            n_frames: None,
        };

        Self {
            stream_spec,
            frequency,
            n_read: 0,
        }
    }
}

impl SampleReader<i32> for SineGenerator {
    fn stream_spec(&self) -> &StreamSpec {
        &self.stream_spec
    }

    fn read(&mut self, buffer: &mut [i32]) -> Result<usize, SyphonError> {
        for s in buffer.iter_mut() {
            let t = self.n_read as f32 / self.stream_spec.sample_rate as f32;
            *s = (t * self.frequency * 2.0 * std::f32::consts::PI).sin() as i32;
        }

        self.n_read += buffer.len();
        Ok(buffer.len())
    }

    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError> {
        match offset {
            SeekFrom::Current(n) => {
                let new_pos = self.n_read as i64 + n;
                if new_pos < 0 {
                    return Err(SyphonError::BadRequest);
                }

                self.n_read = new_pos as usize;
                Ok(self.n_read as u64)
            }
            SeekFrom::Start(n) => {
                self.n_read = n as usize;
                Ok(n)
            }
            SeekFrom::End(n) => Err(SyphonError::Unsupported),
        }
    }
}

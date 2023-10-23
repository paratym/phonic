use crate::{Sample, SampleReader, SampleStream, StreamSpec, SyphonError};
use std::marker::PhantomData;

pub struct NFramesAdapter<T, S: Sample> {
    stream: T,
    spec: StreamSpec,
    n_frames: u64,
    i: usize,
    _sample_type: PhantomData<S>,
}

impl<T, S: Sample> NFramesAdapter<T, S> {
    pub fn from_stream(stream: T, n_frames: u64) -> Self
    where
        T: SampleStream<S>,
    {
        let spec = StreamSpec {
            n_frames: Some(n_frames),
            ..*stream.spec()
        };

        Self {
            stream,
            spec,
            n_frames,
            i: 0,
            _sample_type: PhantomData,
        }
    }
}

impl<T, S: Sample> SampleStream<S> for NFramesAdapter<T, S> {
    fn spec(&self) -> &StreamSpec {
        &self.spec
    }
}

impl<T: SampleReader<S>, S: Sample> SampleReader<S> for NFramesAdapter<T, S> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        let n_channels = self.spec.n_channels as usize;
        let buf_len = buffer
            .len()
            .min((self.n_frames as usize - self.i) * n_channels);

        let n_read = match self.stream.read(&mut buffer[..buf_len]) {
            Ok(0) => {
                buffer[..buf_len].fill(S::MID);
                buf_len
            }
            Ok(n) if n % n_channels != 0 => todo!(),
            Ok(n) => n,
            Err(err) => return Err(err),
        };

        self.i += n_read / n_channels;
        Ok(n_read)
    }
}

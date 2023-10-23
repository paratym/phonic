use crate::{FromSample, Sample, SampleReader, SampleStream, StreamSpec, SyphonError};
use std::marker::PhantomData;

pub struct SampleTypeAdapter<T, S: Sample, O: Sample + FromSample<S>, const N: usize> {
    inner: T,
    buffer: [S; N],
    spec: StreamSpec,
    _sample_type: PhantomData<O>,
}

impl<const N: usize, T, S: Sample, O: Sample + FromSample<S>> SampleTypeAdapter<T, S, O, N> {
    fn new(inner: T, spec: StreamSpec) -> Self {
        Self {
            inner,
            buffer: [S::MID; N],
            spec: StreamSpec {
                sample_format: O::FORMAT,
                ..spec
            },
            _sample_type: PhantomData,
        }
    }

    pub fn reader(inner: T) -> Self
    where
        T: SampleReader<S>,
    {
        let spec = *inner.spec();
        Self::new(inner, spec)
    }
}

impl<const N: usize, T, S: Sample, O: Sample + FromSample<S>> SampleStream<O>
    for SampleTypeAdapter<T, S, O, N>
{
    fn spec(&self) -> &StreamSpec {
        &self.spec
    }
}

impl<const N: usize, T, S, O> SampleReader<O> for SampleTypeAdapter<T, S, O, N>
where
    T: SampleReader<S>,
    S: Sample,
    O: Sample + FromSample<S>,
{
    fn read(&mut self, buffer: &mut [O]) -> Result<usize, SyphonError> {
        let mut buf_len = buffer.len().min(self.buffer.len());
        buf_len -= buf_len % self.spec.block_size;

        let n_read = match self.inner.read(&mut self.buffer[..buf_len]) {
            Ok(0) => return Ok(0),
            Ok(n) if n % self.spec.block_size == 0 => n,
            Ok(_) => return Err(SyphonError::StreamMismatch),
            Err(err) => return Err(err),
        };

        for (in_sample, out_sample) in self.buffer.iter().zip(buffer.iter_mut()) {
            *out_sample = O::from(*in_sample);
        }

        Ok(n_read)
    }
}

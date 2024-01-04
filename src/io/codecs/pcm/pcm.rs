use crate::{
    io::{StreamSpec, SyphonCodec},
    KnownSample, Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError,
};
use byte_slice_cast::{AsByteSlice, AsMutByteSlice, ToByteSlice, ToMutByteSlice};
use std::io::{Read, Write};

pub fn fill_pcm_stream_spec(spec: &mut StreamSpec) -> Result<(), SyphonError> {
    if spec.codec.get_or_insert(SyphonCodec::Pcm) != &SyphonCodec::Pcm {
        return Err(SyphonError::InvalidData);
    }

    spec.set_compression_ratio(1.0)
}

pub struct PcmCodec<T> {
    inner: T,
    spec: SignalSpec,
}

impl<T> PcmCodec<T> {
    pub fn new(inner: T, spec: SignalSpec) -> Self {
        Self { inner, spec }
    }
}

impl<T> Signal for PcmCodec<T> {
    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T, S> SignalReader<S> for PcmCodec<T>
where
    T: Read,
    S: KnownSample + ToMutByteSlice,
{
    fn read(&mut self, buf: &mut [S]) -> Result<usize, SyphonError> {
        let byte_buf = buf.as_mut_byte_slice();
        let n = self.inner.read(byte_buf)?;

        let bytes_per_sample = byte_buf.len() / buf.len();
        if n % bytes_per_sample != 0 {
            todo!()
        }

        Ok(n / bytes_per_sample)
    }
}

impl<T, S> SignalWriter<S> for PcmCodec<T>
where
    T: Write,
    S: Sample + ToByteSlice,
{
    fn write(&mut self, buf: &[S]) -> Result<usize, SyphonError> {
        let byte_buf = buf.as_byte_slice();
        let n = self.inner.write(byte_buf)?;

        let bytes_per_sample = byte_buf.len() / buf.len();
        if n % bytes_per_sample != 0 {
            todo!()
        }

        Ok(n / bytes_per_sample)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.inner.flush().map_err(Into::into)
    }
}

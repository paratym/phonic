use crate::{
    io::{Stream, StreamReader, StreamWriter},
    KnownSample, Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError,
};
use byte_slice_cast::{AsByteSlice, AsMutByteSlice, ToByteSlice, ToMutByteSlice};

pub struct PcmCodec<T: Stream> {
    stream: T,
}

impl<T: Stream> PcmCodec<T> {
    pub fn new(stream: T) -> Result<Self, SyphonError> {
        let spec = stream.spec();

        let calculated_bytes_per_frame = spec
            .byte_len
            .zip(spec.decoded_spec.n_frames)
            .map(|(n, f)| n / f as u64);

        let actual_bytes_per_frame = spec.decoded_spec.sample_type.byte_size() as u64
            * spec.decoded_spec.channels.count() as u64;

        if calculated_bytes_per_frame.is_some_and(|c| c != actual_bytes_per_frame) {
            return Err(SyphonError::Unsupported);
        }

        Ok(Self { stream })
    }
}

impl<T: Stream> Signal for PcmCodec<T> {
    fn spec(&self) -> &SignalSpec {
        &self.stream.spec().decoded_spec
    }
}

impl<T: StreamReader, S: KnownSample + ToMutByteSlice> SignalReader<S> for PcmCodec<T> {
    fn read(&mut self, buf: &mut [S]) -> Result<usize, SyphonError> {
        let byte_buf = buf.as_mut_byte_slice();
        let n = self.stream.read(byte_buf)?;

        let bytes_per_sample = byte_buf.len() / buf.len();
        if n % bytes_per_sample != 0 {
            todo!()
        }

        Ok(n)
    }
}

impl<T: StreamWriter, S: Sample + ToByteSlice> SignalWriter<S> for PcmCodec<T> {
    fn write(&mut self, buf: &[S]) -> Result<usize, SyphonError> {
        let byte_buf = buf.as_byte_slice();
        let n = self.stream.write(byte_buf)?;

        let bytes_per_sample = byte_buf.len() / buf.len();
        if n % bytes_per_sample != 0 {
            todo!()
        }

        Ok(n)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.stream.flush()
    }
}

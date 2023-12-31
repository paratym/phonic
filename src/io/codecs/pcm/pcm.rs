use std::{rc::Rc, cell::RefCell};
use crate::{
    io::{TaggedSignalReader, TaggedSignalWriter, Stream, StreamReader, StreamSpecBuilder, StreamWriter},
    Sample, SampleType, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError, KnownSample,
};
use byte_slice_cast::{AsByteSlice, AsMutByteSlice, ToByteSlice, ToMutByteSlice};

pub struct PcmCodec<T: Stream> {
    stream: T,
}

pub fn fill_pcm_spec(spec: &mut StreamSpecBuilder) -> Result<(), SyphonError> {
    // if spec.decoded_spec.block_size.is_none() {
    //     spec.decoded_spec.block_size = spec
    //         .block_size
    //         .zip(spec.decoded_spec.sample_type)
    //         .zip(spec.decoded_spec.channels)
    //         .map(|((b, s), c)| b / s.byte_size() / c.count() as usize)
    //         .or(Some(1));
    // }

    // let bytes_per_decoded_block = spec
    //     .decoded_spec
    //     .samples_per_block()
    //     .zip(spec.decoded_spec.sample_type)
    //     .map(|(n, s)| n * s.byte_size());

    // if spec.block_size.is_none() {
    //     spec.block_size = bytes_per_decoded_block
    // }

    // if bytes_per_decoded_block
    //     .zip(spec.block_size)
    //     .map_or(false, |(d, e)| d % e != 0)
    // {
    //     return Err(SyphonError::Unsupported);
    // }

    // if spec.byte_len.is_none() {
    //     spec.byte_len = bytes_per_decoded_block
    //         .zip(spec.decoded_spec.n_blocks)
    //         .map(|(b, n)| n * b as u64);
    // } else if spec.decoded_spec.n_blocks.is_none() {
    //     spec.decoded_spec.n_blocks = bytes_per_decoded_block
    //         .zip(spec.byte_len)
    //         .map(|(b, n)| n / b as u64);
    // }

    Ok(())
}

impl<T: Stream> PcmCodec<T> {
    pub fn new(inner: T) -> Result<Self, SyphonError> {
        Ok(Self { stream: inner })
    }
}

impl<T: Stream> Signal for PcmCodec<T> {
    fn spec(&self) -> &SignalSpec {
        &self.stream.spec().decoded_spec
    }
}

impl<T: StreamReader, S: Sample + ToMutByteSlice> SignalReader<S> for PcmCodec<T> {
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
}

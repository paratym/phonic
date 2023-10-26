use crate::{
    io::{EncodedStream, EncodedStreamSpecBuilder, SignalReaderRef, SignalWriterRef},
    Sample, SampleFormat, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError,
};
use byte_slice_cast::{AsByteSlice, AsMutByteSlice, ToByteSlice, ToMutByteSlice};
use std::io::{Read, Seek, SeekFrom, Write};

pub struct PcmCodec<T> {
    inner: T,
    spec: SignalSpec,
}

pub fn fill_pcm_spec(spec: &mut EncodedStreamSpecBuilder) -> Result<(), SyphonError> {
    if spec.decoded_spec.block_size.is_none() {
        spec.decoded_spec.block_size = spec
            .block_size
            .zip(spec.decoded_spec.sample_format)
            .zip(spec.decoded_spec.n_channels)
            .map(|((b, s), c)| b / s.byte_size() / c as usize)
            .or(Some(1));
    }

    let bytes_per_decoded_block = spec
        .decoded_spec
        .samples_per_block()
        .zip(spec.decoded_spec.sample_format)
        .map(|(n, s)| n * s.byte_size());

    if bytes_per_decoded_block
        .zip(spec.decoded_spec.sample_format)
        .map_or(false, |(b, s)| b % s.byte_size() != 0)
    {
        return Err(SyphonError::Unsupported);
    }

    if spec.block_size.is_none() {
        spec.block_size = bytes_per_decoded_block
    }

    if bytes_per_decoded_block
        .zip(spec.block_size)
        .map_or(false, |(d, e)| d % e != 0)
    {
        return Err(SyphonError::Unsupported);
    }

    if spec.byte_len.is_none() {
        spec.byte_len = bytes_per_decoded_block
            .zip(spec.decoded_spec.n_blocks)
            .map(|(b, n)| n * b as u64);
    } else if spec.decoded_spec.n_blocks.is_none() {
        spec.decoded_spec.n_blocks = bytes_per_decoded_block
            .zip(spec.byte_len)
            .map(|(b, n)| n / b as u64);
    }

    Ok(())
}

impl<T> PcmCodec<T> {
    pub fn from_stream(inner: T) -> Result<Self, SyphonError>
    where
        T: EncodedStream,
    {
        let mut spec = inner.spec().clone().into();
        fill_pcm_spec(&mut spec)?;

        Ok(Self {
            inner,
            spec: spec.decoded_spec.build()?,
        })
    }
}

impl<T> Signal for PcmCodec<T> {
    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: Read, S: Sample + ToMutByteSlice> SignalReader<S> for PcmCodec<T> {
    fn read(&mut self, buf: &mut [S]) -> Result<usize, SyphonError> {
        if S::FORMAT != self.spec.sample_format {
            return Err(SyphonError::InvalidData);
        }

        let mut buf_len = buf.len();
        buf_len -= buf_len % self.spec.samples_per_block();

        let bytes_per_block = self.spec.samples_per_block() * S::FORMAT.byte_size();
        match self.inner.read(buf[..buf_len].as_mut_byte_slice()) {
            Ok(n) if n % bytes_per_block == 0 => Ok(n / bytes_per_block),
            Ok(_) => todo!(),
            Err(e) => Err(e.into()),
        }
    }
}

impl<T: Write, S: Sample + ToByteSlice> SignalWriter<S> for PcmCodec<T> {
    fn write(&mut self, buf: &[S]) -> Result<usize, SyphonError> {
        if S::FORMAT != self.spec.sample_format {
            return Err(SyphonError::InvalidData);
        }

        let mut buf_len = buf.len();
        buf_len -= buf_len % self.spec.samples_per_block();

        let bytes_per_block = self.spec.samples_per_block() * S::FORMAT.byte_size();
        match self.inner.write(buf[..buf_len].as_byte_slice()) {
            Ok(n) if n % bytes_per_block == 0 => Ok(n / bytes_per_block),
            Ok(_) => todo!(),
            Err(e) => Err(e.into()),
        }
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        Ok(self.inner.flush()?)
    }
}

impl<T: Read + 'static> From<PcmCodec<T>> for SignalReaderRef {
    fn from(decoder: PcmCodec<T>) -> Self {
        match decoder.spec.sample_format {
            SampleFormat::U8 => Self::U8(Box::new(decoder)),
            SampleFormat::U16 => Self::U16(Box::new(decoder)),
            SampleFormat::U32 => Self::U32(Box::new(decoder)),
            SampleFormat::U64 => Self::U64(Box::new(decoder)),

            SampleFormat::I8 => Self::I8(Box::new(decoder)),
            SampleFormat::I16 => Self::I16(Box::new(decoder)),
            SampleFormat::I32 => Self::I32(Box::new(decoder)),
            SampleFormat::I64 => Self::I64(Box::new(decoder)),

            SampleFormat::F32 => Self::F32(Box::new(decoder)),
            SampleFormat::F64 => Self::F64(Box::new(decoder)),
        }
    }
}

impl<T: Write + 'static> From<PcmCodec<T>> for SignalWriterRef {
    fn from(encoder: PcmCodec<T>) -> Self {
        match encoder.spec.sample_format {
            SampleFormat::U8 => Self::U8(Box::new(encoder)),
            SampleFormat::U16 => Self::U16(Box::new(encoder)),
            SampleFormat::U32 => Self::U32(Box::new(encoder)),
            SampleFormat::U64 => Self::U64(Box::new(encoder)),

            SampleFormat::I8 => Self::I8(Box::new(encoder)),
            SampleFormat::I16 => Self::I16(Box::new(encoder)),
            SampleFormat::I32 => Self::I32(Box::new(encoder)),
            SampleFormat::I64 => Self::I64(Box::new(encoder)),

            SampleFormat::F32 => Self::F32(Box::new(encoder)),
            SampleFormat::F64 => Self::F64(Box::new(encoder)),
        }
    }
}

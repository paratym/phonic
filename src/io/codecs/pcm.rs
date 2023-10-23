use crate::{
    io::{EncodedStream, EncodedStreamSpecBuilder, SampleReaderRef, SampleWriterRef},
    Sample, SampleFormat, SampleReader, SampleStream, SampleWriter, StreamSpec, SyphonError,
};
use byte_slice_cast::{AsByteSlice, AsMutByteSlice, ToByteSlice, ToMutByteSlice};
use std::{
    io::{Read, Seek, SeekFrom, Write},
    mem::size_of,
};

pub struct PcmCodec<T> {
    inner: T,
    spec: StreamSpec,
}

impl<T> PcmCodec<T> {
    pub fn fill_spec(spec: &mut EncodedStreamSpecBuilder) -> Result<(), SyphonError> {
        if spec.block_size.is_none() {
            spec.block_size = spec
                .decoded_spec
                .block_size
                .or(spec.decoded_spec.n_channels.map(usize::from))
                .zip(spec.decoded_spec.sample_format)
                .map(|(n, s)| n * s.byte_size());
        }

        if spec
            .block_size
            .zip(spec.decoded_spec.sample_format)
            .map_or(false, |(b, s)| b % s.byte_size() != 0)
        {
            return Err(SyphonError::InvalidInput);
        }

        if spec.decoded_spec.block_size.is_none() {
            spec.decoded_spec.block_size = spec
                .block_size
                .zip(spec.decoded_spec.sample_format)
                .map(|(n, s)| n / s.byte_size())
                .or(spec.decoded_spec.n_channels.map(usize::from));
        }

        let bytes_per_decoded_block = spec
            .decoded_spec
            .block_size
            .zip(spec.decoded_spec.sample_format)
            .map(|(n, s)| n * s.byte_size());

        if bytes_per_decoded_block
            .zip(spec.decoded_spec.sample_format)
            .map_or(false, |(b, s)| b % s.byte_size() != 0)
        {
            return Err(SyphonError::InvalidInput);
        }

        if bytes_per_decoded_block
            .zip(spec.block_size)
            .map_or(false, |(d, e)| d % e != 0)
        {
            return Err(SyphonError::InvalidInput);
        }

        let bytes_per_frame = spec
            .decoded_spec
            .n_channels
            .zip(spec.decoded_spec.sample_format)
            .map(|(n, s)| n as usize * s.byte_size());

        if spec.byte_len.is_none() {
            spec.byte_len = bytes_per_frame
                .zip(spec.decoded_spec.n_frames)
                .map(|(b, n)| n * b as u64);
        } else if spec.decoded_spec.n_frames.is_none() {
            spec.decoded_spec.n_frames = bytes_per_frame
                .zip(spec.byte_len)
                .map(|(b, n)| n / b as u64);
        }

        Ok(())
    }

    pub fn from_stream(inner: T) -> Result<Self, SyphonError>
    where
        T: EncodedStream,
    {
        let mut spec = inner.spec().clone().into();
        Self::fill_spec(&mut spec)?;

        Ok(Self {
            inner,
            spec: spec.decoded_spec.build()?,
        })
    }

    pub fn read<S: Sample + ToMutByteSlice>(&mut self, buf: &mut [S]) -> Result<usize, SyphonError>
    where
        T: Read,
    {
        if S::FORMAT != self.spec.sample_format {
            return Err(SyphonError::InvalidData);
        }

        let n_samples = buf.len() - (buf.len() % self.spec.block_size);
        let sample_block_size = self.spec.block_size * size_of::<S>();

        match self.inner.read(buf[..n_samples].as_mut_byte_slice()) {
            Ok(n) if n % sample_block_size == 0 => Ok(n / size_of::<S>()),
            Ok(_) => todo!(),
            Err(e) => Err(e.into()),
        }
    }

    pub fn write<S: Sample + ToByteSlice>(&mut self, buf: &[S]) -> Result<usize, SyphonError>
    where
        T: Write,
    {
        if S::FORMAT != self.spec.sample_format {
            return Err(SyphonError::InvalidData);
        }

        let n_samples = buf.len() - (buf.len() % self.spec.block_size);
        let sample_block_size = self.spec.block_size * size_of::<S>();

        match self.inner.write(buf[..n_samples].as_byte_slice()) {
            Ok(n) if n % sample_block_size == 0 => Ok(n / size_of::<S>()),
            Ok(_) => todo!(),
            Err(e) => Err(e.into()),
        }
    }
}

impl<T: Seek> Seek for PcmCodec<T> {
    fn seek(&mut self, offset: SeekFrom) -> std::io::Result<u64> {
        todo!()
    }
}

impl<T, S: Sample> SampleStream<S> for PcmCodec<T> {
    fn spec(&self) -> &StreamSpec {
        &self.spec
    }
}

impl<T: Read, S: Sample + ToMutByteSlice> SampleReader<S> for PcmCodec<T> {
    fn read(&mut self, buf: &mut [S]) -> Result<usize, SyphonError> {
        self.read(buf)
    }
}

impl<T: Write, S: Sample + ToByteSlice> SampleWriter<S> for PcmCodec<T> {
    fn write(&mut self, buf: &[S]) -> Result<usize, SyphonError> {
        self.write(buf)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        Ok(self.inner.flush()?)
    }
}

impl<T: Read + 'static> From<PcmCodec<T>> for SampleReaderRef {
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

impl<T: Write + 'static> From<PcmCodec<T>> for SampleWriterRef {
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

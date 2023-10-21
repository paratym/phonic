use crate::{
    io::{
        EncodedStreamReader, EncodedStreamSpec, EncodedStreamWriter, SampleReader, SampleReaderRef,
        SampleWriter, SampleWriterRef, StreamSpec,
    },
    Sample, SampleFormat, SyphonError,
};
use byte_slice_cast::{AsByteSlice, AsMutByteSlice, ToByteSlice, ToMutByteSlice};
use std::{
    io::{Read, Seek, SeekFrom, Write},
    mem::size_of,
};

pub struct PcmCodec<T> {
    inner: T,
    stream_spec: StreamSpec,
}

impl<T> PcmCodec<T> {
    pub fn new(inner: T, stream_spec: StreamSpec) -> Self {
        Self { inner, stream_spec }
    }

    pub fn from_encoded_spec(
        inner: T,
        mut encoded_spec: EncodedStreamSpec,
    ) -> Result<Self, SyphonError> {
        if encoded_spec.decoded_spec.block_size.is_none() {
            encoded_spec.decoded_spec.block_size =
                encoded_spec.decoded_spec.n_channels.map(|n| n as usize);
        }

        let mut stream_spec = encoded_spec.decoded_spec.try_build()?;
        if stream_spec.bytes_per_block() % encoded_spec.block_size != 0 {
            return Err(SyphonError::BadRequest);
        }

        if stream_spec.n_frames.is_none() {
            if let Some(byte_len) = encoded_spec.byte_len {
                let bytes_per_frame =
                    stream_spec.n_channels as u64 * stream_spec.sample_format.byte_size() as u64;

                stream_spec.n_frames = Some(byte_len / bytes_per_frame);
            }
        }

        Ok(Self { inner, stream_spec })
    }

    pub fn decoder(inner: T) -> Result<Self, SyphonError>
    where
        T: EncodedStreamReader,
    {
        let encoded_spec = *inner.stream_spec();
        Self::from_encoded_spec(inner, encoded_spec)
    }

    pub fn encoder(inner: T) -> Result<Self, SyphonError>
    where
        T: EncodedStreamWriter,
    {
        let encoded_spec = *inner.stream_spec();
        Self::from_encoded_spec(inner, encoded_spec)
    }

    pub fn read<S: Sample + ToMutByteSlice>(&mut self, buf: &mut [S]) -> Result<usize, SyphonError>
    where
        T: Read,
    {
        if S::FORMAT != self.stream_spec.sample_format {
            return Err(SyphonError::StreamMismatch);
        }

        let n_samples = buf.len() - (buf.len() % self.stream_spec.block_size);
        let sample_block_size = self.stream_spec.block_size * size_of::<S>();

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
        if S::FORMAT != self.stream_spec.sample_format {
            return Err(SyphonError::StreamMismatch);
        }

        let n_samples = buf.len() - (buf.len() % self.stream_spec.block_size);
        let sample_block_size = self.stream_spec.block_size * size_of::<S>();

        match self.inner.write(buf[..n_samples].as_byte_slice()) {
            Ok(n) if n % sample_block_size == 0 => Ok(n / size_of::<S>()),
            Ok(_) => todo!(),
            Err(e) => Err(e.into()),
        }
    }

    pub fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError>
    where
        T: Seek,
    {
        todo!();
    }
}

impl<T: Read + Seek, S: Sample + ToMutByteSlice> SampleReader<S> for PcmCodec<T> {
    fn stream_spec(&self) -> &StreamSpec {
        &self.stream_spec
    }

    fn read(&mut self, buf: &mut [S]) -> Result<usize, SyphonError> {
        self.read(buf)
    }

    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError> {
        self.seek(offset)
    }
}

impl<T: Write + Seek, S: Sample + ToByteSlice> SampleWriter<S> for PcmCodec<T> {
    fn stream_spec(&self) -> &StreamSpec {
        &self.stream_spec
    }

    fn write(&mut self, buf: &[S]) -> Result<usize, SyphonError> {
        self.write(buf)
    }

    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError> {
        self.seek(offset)
    }
}

impl<T: Read + Seek + 'static> From<PcmCodec<T>> for SampleReaderRef {
    fn from(decoder: PcmCodec<T>) -> Self {
        match decoder.stream_spec.sample_format {
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

impl<T: Write + Seek + 'static> From<PcmCodec<T>> for SampleWriterRef {
    fn from(encoder: PcmCodec<T>) -> Self {
        match encoder.stream_spec.sample_format {
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

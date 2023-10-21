use crate::{
    io::{
        EncodedStreamReader, EncodedStreamSpec, SampleReader, SampleReaderRef, SampleWriter,
        StreamSpec,
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

        Ok(Self::new(inner, stream_spec))
    }

    pub fn from_encoded_stream(inner: T) -> Result<Self, SyphonError>
    where
        T: EncodedStreamReader,
    {
        let spec = *inner.stream_spec();
        Self::from_encoded_spec(inner, spec)
    }

    pub fn into_sample_reader_ref(self) -> SampleReaderRef
    where
        T: Read + Seek + 'static,
    {
        let decoder_ref = Box::new(self);
        match decoder_ref.stream_spec.sample_format {
            SampleFormat::U8 => SampleReaderRef::U8(decoder_ref),
            SampleFormat::U16 => SampleReaderRef::U16(decoder_ref),
            SampleFormat::U32 => SampleReaderRef::U32(decoder_ref),
            SampleFormat::U64 => SampleReaderRef::U64(decoder_ref),

            SampleFormat::I8 => SampleReaderRef::I8(decoder_ref),
            SampleFormat::I16 => SampleReaderRef::I16(decoder_ref),
            SampleFormat::I32 => SampleReaderRef::I32(decoder_ref),
            SampleFormat::I64 => SampleReaderRef::I64(decoder_ref),

            SampleFormat::F32 => SampleReaderRef::F32(decoder_ref),
            SampleFormat::F64 => SampleReaderRef::F64(decoder_ref),
        }
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

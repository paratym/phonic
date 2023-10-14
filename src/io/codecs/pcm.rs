use crate::{
    io::{MediaStreamReader, SampleReader, SampleReaderRef, SampleWriter, StreamSpec},
    Sample, SampleFormat, SyphonError,
};
use byte_slice_cast::{AsByteSlice, AsMutByteSlice, ToByteSlice, ToMutByteSlice};
use std::io::{Read, Write};

pub struct PcmDecoder {
    reader: Box<dyn MediaStreamReader>,
}

impl PcmDecoder {
    pub fn new(reader: Box<dyn MediaStreamReader>) -> Self {
        Self { reader }
    }

    pub fn try_into_sample_reader_ref(self) -> Result<SampleReaderRef, SyphonError> {
        let decoder_ref = Box::new(self);
        let spec = decoder_ref.reader.stream_spec();
        match (spec.sample_format, spec.bytes_per_sample) {
            (SampleFormat::Unsigned(_), 1) => Ok(SampleReaderRef::U8(decoder_ref)),
            (SampleFormat::Unsigned(_), 2) => Ok(SampleReaderRef::U16(decoder_ref)),
            (SampleFormat::Unsigned(_), 4) => Ok(SampleReaderRef::U32(decoder_ref)),
            (SampleFormat::Unsigned(_), 8) => Ok(SampleReaderRef::U64(decoder_ref)),

            (SampleFormat::Signed(_), 1) => Ok(SampleReaderRef::I8(decoder_ref)),
            (SampleFormat::Signed(_), 2) => Ok(SampleReaderRef::I16(decoder_ref)),
            (SampleFormat::Signed(_), 4) => Ok(SampleReaderRef::I32(decoder_ref)),
            (SampleFormat::Signed(_), 8) => Ok(SampleReaderRef::I64(decoder_ref)),
            
            (SampleFormat::Float(_), 4) => Ok(SampleReaderRef::F32(decoder_ref)),
            (SampleFormat::Float(_), 8) => Ok(SampleReaderRef::F64(decoder_ref)),

            _ => Err(SyphonError::Unsupported),
        }
    }
}

impl<S: Sample + ToMutByteSlice> SampleReader<S> for PcmDecoder {
    fn stream_spec(&self) -> &StreamSpec {
        self.reader.stream_spec()
    }

    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        let spec = self.reader.stream_spec();
        let n_samples = buffer.len() / spec.block_size * spec.block_size;
        let bytes_per_block = <S>::N_BYTES * spec.block_size;

        match self.reader.read(buffer[..n_samples].as_mut_byte_slice()) {
            Ok(n) if n % bytes_per_block == 0 => Ok(n / <S>::N_BYTES),
            Ok(_) => Err(SyphonError::StreamMismatch),
            Err(e) => Err(e.into()),
        }
    }
}

pub struct PcmEncoder {
    writer: Box<dyn Write>,
    stream_spec: StreamSpec,
}

impl PcmEncoder {
    pub fn new(writer: Box<dyn Write>, stream_spec: StreamSpec) -> Self {
        Self {
            writer,
            stream_spec,
        }
    }
}

impl<S: Sample + ToByteSlice> SampleWriter<S> for PcmEncoder {
    fn stream_spec(&self) -> &StreamSpec {
        &self.stream_spec
    }

    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        let n_samples = buffer.len() / self.stream_spec.block_size * self.stream_spec.block_size;
        let bytes_per_block = <S>::N_BYTES * self.stream_spec.block_size;

        match self.writer.write(buffer[..n_samples].as_byte_slice()) {
            Ok(n) if n % bytes_per_block == 0 => Ok(n / <S>::N_BYTES),
            Ok(_) => Err(SyphonError::StreamMismatch),
            Err(e) => Err(e.into()),
        }
    }
}

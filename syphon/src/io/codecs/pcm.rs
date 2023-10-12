use crate::core::{
    Sample, SampleFormat, SampleReader, SampleReaderRef, SampleWriter, SignalSpec, SyphonError,
};
use byte_slice_cast::{AsByteSlice, AsMutByteSlice, ToByteSlice, ToMutByteSlice};
use std::io::{Read, Write};

pub struct PcmDecoder {
    reader: Box<dyn Read>,
    signal_spec: SignalSpec,
}

impl PcmDecoder {
    pub fn new(reader: Box<dyn Read>, signal_spec: SignalSpec) -> Self {
        Self {
            reader,
            signal_spec,
        }
    }

    pub fn try_into_sample_reader_ref(self) -> Option<SampleReaderRef> {
        let decoder_ref = Box::new(self);
        let reader_ref = match (
            decoder_ref.signal_spec.sample_format,
            decoder_ref.signal_spec.bytes_per_sample,
        ) {
            (SampleFormat::Unsigned(_), 1) => SampleReaderRef::U8(decoder_ref),
            (SampleFormat::Unsigned(_), 2) => SampleReaderRef::U16(decoder_ref),
            (SampleFormat::Unsigned(_), 4) => SampleReaderRef::U32(decoder_ref),
            (SampleFormat::Unsigned(_), 8) => SampleReaderRef::U64(decoder_ref),

            (SampleFormat::Signed(_), 1) => SampleReaderRef::I8(decoder_ref),
            (SampleFormat::Signed(_), 2) => SampleReaderRef::I16(decoder_ref),
            (SampleFormat::Signed(_), 4) => SampleReaderRef::I32(decoder_ref),
            (SampleFormat::Signed(_), 8) => SampleReaderRef::I64(decoder_ref),

            (SampleFormat::Float(_), 4) => SampleReaderRef::F32(decoder_ref),
            (SampleFormat::Float(_), 8) => SampleReaderRef::F64(decoder_ref),

            _ => return None,
        };

        Some(reader_ref)
    }
}

impl<S: Sample + ToMutByteSlice> SampleReader<S> for PcmDecoder {
    fn signal_spec(&self) -> SignalSpec {
        self.signal_spec
    }

    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        let n_samples = buffer.len() / self.signal_spec.block_size * self.signal_spec.block_size;
        let bytes_per_block = <S>::N_BYTES * self.signal_spec.block_size;

        match self.reader.read(buffer[..n_samples].as_mut_byte_slice()) {
            Ok(n) if n % bytes_per_block == 0 => Ok(n / <S>::N_BYTES),
            Ok(_) => Err(SyphonError::SignalMismatch),
            Err(e) => Err(e.into()),
        }
    }
}

pub struct PcmEncoder {
    writer: Box<dyn Write>,
    signal_spec: SignalSpec,
}

impl PcmEncoder {
    pub fn new(writer: Box<dyn Write>, signal_spec: SignalSpec) -> Self {
        Self {
            writer,
            signal_spec,
        }
    }
}

impl<S: Sample + ToByteSlice> SampleWriter<S> for PcmEncoder {
    fn signal_spec(&self) -> SignalSpec {
        self.signal_spec
    }

    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        let n_samples = buffer.len() / self.signal_spec.block_size * self.signal_spec.block_size;
        let bytes_per_block = <S>::N_BYTES * self.signal_spec.block_size;

        match self.writer.write(buffer[..n_samples].as_byte_slice()) {
            Ok(n) if n % bytes_per_block == 0 => Ok(n / <S>::N_BYTES),
            Ok(_) => Err(SyphonError::SignalMismatch),
            Err(e) => Err(e.into()),
        }
    }
}

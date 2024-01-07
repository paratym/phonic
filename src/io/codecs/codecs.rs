use byte_slice_cast::FromByteSlice;

use crate::{
    io::{
        codecs::pcm::{fill_pcm_stream_spec, PcmCodec},
        Stream, StreamReader, StreamSpec, StreamSpecBuilder, StreamWriter, TaggedSignalReader,
        TaggedSignalWriter,
    },
    SignalReader, SignalWriter, SyphonError, KnownSample,
};
use std::{
    any::TypeId,
    io::{Read, Write},
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SyphonCodec {
    Pcm,
}

impl SyphonCodec {
    pub fn fill_spec(spec: StreamSpecBuilder) -> Result<StreamSpecBuilder, SyphonError> {
        match spec.codec.ok_or(SyphonError::MissingData)? {
            Self::Pcm => fill_pcm_stream_spec(spec),
        }
    }

    pub fn construct_decoder_reader(
        reader: impl Read + 'static,
        spec: StreamSpecBuilder,
    ) -> Result<TaggedSignalReader, SyphonError> {
        let sample_type = spec.sample_type.ok_or(SyphonError::MissingData)?;

        Ok(match spec.codec.ok_or(SyphonError::MissingData)? {
            Self::Pcm => {
                if sample_type == TypeId::of::<i8>() {
                    TaggedSignalReader::I8(Box::new(PcmCodec::new(reader, spec)?))
                } else if sample_type == TypeId::of::<i16>() {
                    TaggedSignalReader::I16(Box::new(PcmCodec::new(reader, spec)?))
                } else if sample_type == TypeId::of::<i32>() {
                    TaggedSignalReader::I32(Box::new(PcmCodec::new(reader, spec)?))
                } else if sample_type == TypeId::of::<i64>() {
                    TaggedSignalReader::I64(Box::new(PcmCodec::new(reader, spec)?))
                } else if sample_type == TypeId::of::<u8>() {
                    TaggedSignalReader::U8(Box::new(PcmCodec::new(reader, spec)?))
                } else if sample_type == TypeId::of::<u16>() {
                    TaggedSignalReader::U16(Box::new(PcmCodec::new(reader, spec)?))
                } else if sample_type == TypeId::of::<u32>() {
                    TaggedSignalReader::U32(Box::new(PcmCodec::new(reader, spec)?))
                } else if sample_type == TypeId::of::<u64>() {
                    TaggedSignalReader::U64(Box::new(PcmCodec::new(reader, spec)?))
                } else if sample_type == TypeId::of::<f32>() {
                    TaggedSignalReader::F32(Box::new(PcmCodec::new(reader, spec)?))
                } else if sample_type == TypeId::of::<f64>() {
                    TaggedSignalReader::F64(Box::new(PcmCodec::new(reader, spec)?))
                } else {
                    return Err(SyphonError::Unsupported);
                }
            }
        })
    }

    pub fn construct_encoder_writer(
        writer: impl Write + 'static,
        spec: StreamSpecBuilder,
    ) -> Result<TaggedSignalWriter, SyphonError> {
        let sample_type = spec.sample_type.ok_or(SyphonError::MissingData)?;

        Ok(match spec.codec.ok_or(SyphonError::MissingData)? {
            Self::Pcm => {
                if sample_type == TypeId::of::<i8>() {
                    TaggedSignalWriter::I8(Box::new(PcmCodec::new(writer, spec)?))
                } else if sample_type == TypeId::of::<i16>() {
                    TaggedSignalWriter::I16(Box::new(PcmCodec::new(writer, spec)?))
                } else if sample_type == TypeId::of::<i32>() {
                    TaggedSignalWriter::I32(Box::new(PcmCodec::new(writer, spec)?))
                } else if sample_type == TypeId::of::<i64>() {
                    TaggedSignalWriter::I64(Box::new(PcmCodec::new(writer, spec)?))
                } else if sample_type == TypeId::of::<u8>() {
                    TaggedSignalWriter::U8(Box::new(PcmCodec::new(writer, spec)?))
                } else if sample_type == TypeId::of::<u16>() {
                    TaggedSignalWriter::U16(Box::new(PcmCodec::new(writer, spec)?))
                } else if sample_type == TypeId::of::<u32>() {
                    TaggedSignalWriter::U32(Box::new(PcmCodec::new(writer, spec)?))
                } else if sample_type == TypeId::of::<u64>() {
                    TaggedSignalWriter::U64(Box::new(PcmCodec::new(writer, spec)?))
                } else if sample_type == TypeId::of::<f32>() {
                    TaggedSignalWriter::F32(Box::new(PcmCodec::new(writer, spec)?))
                } else if sample_type == TypeId::of::<f64>() {
                    TaggedSignalWriter::F64(Box::new(PcmCodec::new(writer, spec)?))
                } else {
                    return Err(SyphonError::Unsupported);
                }
            }
        })
    }

    pub fn construct_encoder_reader<T>(
        &self,
        reader: T,
    ) -> Result<Box<dyn StreamReader>, SyphonError>
    where
        T: SignalReader + 'static,
        T::Sample: FromByteSlice
    {
        let spec = StreamSpec::builder()
            .with_codec(*self)
            .with_sample_type::<T::Sample>()
            .with_decoded_spec(reader.spec().clone().into());

        Ok(match self {
            Self::Pcm => Box::new(PcmCodec::new(reader, spec)?),
        })
    }

    pub fn construct_decoder_writer<T>(
        &self,
        writer: T,
    ) -> Result<Box<dyn StreamWriter>, SyphonError>
    where
        T: SignalWriter + 'static,
        T::Sample: FromByteSlice
    {
        let spec = StreamSpec::builder()
            .with_codec(*self)
            .with_sample_type::<T::Sample>()
            .with_decoded_spec(writer.spec().clone().into());

        Ok(match self {
            Self::Pcm => Box::new(PcmCodec::new(writer, spec)?),
        })
    }
}

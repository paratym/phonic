use crate::{
    io::{
        codecs::pcm::{fill_pcm_stream_spec, PcmCodec},
        StreamSpec, TaggedSignalReader, TaggedSignalWriter,
    },
    SampleType, SyphonError,
};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SyphonCodec {
    Pcm,
}

impl SyphonCodec {
    pub fn fill_spec(spec: &mut StreamSpec) -> Result<(), SyphonError> {
        match spec.codec.ok_or(SyphonError::MissingData)? {
            Self::Pcm => fill_pcm_stream_spec(spec),
        }
    }

    pub fn construct_decoder(
        &self,
        reader: impl Read + 'static,
        spec: StreamSpec,
    ) -> Result<TaggedSignalReader, SyphonError> {
        let sample_type = spec.sample_type.ok_or(SyphonError::MissingData)?;

        Ok(match self {
            Self::Pcm => match sample_type {
                SampleType::I8 => TaggedSignalReader::I8(Box::new(PcmCodec::new(
                    reader,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::I16 => TaggedSignalReader::I16(Box::new(PcmCodec::new(
                    reader,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::I32 => TaggedSignalReader::I32(Box::new(PcmCodec::new(
                    reader,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::I64 => TaggedSignalReader::I64(Box::new(PcmCodec::new(
                    reader,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::U8 => TaggedSignalReader::U8(Box::new(PcmCodec::new(
                    reader,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::U16 => TaggedSignalReader::U16(Box::new(PcmCodec::new(
                    reader,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::U32 => TaggedSignalReader::U32(Box::new(PcmCodec::new(
                    reader,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::U64 => TaggedSignalReader::U64(Box::new(PcmCodec::new(
                    reader,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::F32 => TaggedSignalReader::F32(Box::new(PcmCodec::new(
                    reader,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::F64 => TaggedSignalReader::F64(Box::new(PcmCodec::new(
                    reader,
                    spec.decoded_spec.build()?,
                ))),
            },
        })
    }

    pub fn construct_encoder(
        &self,
        writer: impl Write + 'static,
        spec: StreamSpec,
    ) -> Result<TaggedSignalWriter, SyphonError> {
        let sample_type = spec.sample_type.ok_or(SyphonError::MissingData)?;

        Ok(match self {
            Self::Pcm => match sample_type {
                SampleType::I8 => TaggedSignalWriter::I8(Box::new(PcmCodec::new(
                    writer,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::I16 => TaggedSignalWriter::I16(Box::new(PcmCodec::new(
                    writer,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::I32 => TaggedSignalWriter::I32(Box::new(PcmCodec::new(
                    writer,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::I64 => TaggedSignalWriter::I64(Box::new(PcmCodec::new(
                    writer,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::U8 => TaggedSignalWriter::U8(Box::new(PcmCodec::new(
                    writer,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::U16 => TaggedSignalWriter::U16(Box::new(PcmCodec::new(
                    writer,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::U32 => TaggedSignalWriter::U32(Box::new(PcmCodec::new(
                    writer,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::U64 => TaggedSignalWriter::U64(Box::new(PcmCodec::new(
                    writer,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::F32 => TaggedSignalWriter::F32(Box::new(PcmCodec::new(
                    writer,
                    spec.decoded_spec.build()?,
                ))),
                SampleType::F64 => TaggedSignalWriter::F64(Box::new(PcmCodec::new(
                    writer,
                    spec.decoded_spec.build()?,
                ))),
            },
        })
    }
}

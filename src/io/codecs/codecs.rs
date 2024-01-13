use crate::{
    io::{
        codecs::pcm::{fill_pcm_stream_spec, PcmCodec},
        KnownSampleType, StreamReader, StreamSpec, StreamWriter,
        TaggedSignalReader, TaggedSignalWriter, Stream,
    },
    signal::{SignalReader, SignalWriter},
    SyphonError,
};
use byte_slice_cast::FromByteSlice;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SyphonCodec {
    Pcm,
}

impl SyphonCodec {
    pub fn fill_spec(spec: StreamSpec) -> Result<StreamSpec, SyphonError> {
        match spec.codec.ok_or(SyphonError::MissingData)? {
            Self::Pcm => fill_pcm_stream_spec(spec),
        }
    }

    pub fn construct_decoder_reader(
        reader: impl Stream + Read + 'static,
    ) -> Result<TaggedSignalReader, SyphonError> {
        let spec = reader.spec();
        let sample_type = spec
            .sample_type
            .ok_or(SyphonError::MissingData)?
            .try_into()?;

        Ok(match spec.codec.ok_or(SyphonError::MissingData)? {
            Self::Pcm => match sample_type {
                KnownSampleType::I8 => {
                    TaggedSignalReader::I8(Box::new(PcmCodec::from_stream(reader)?))
                }
                KnownSampleType::I16 => {
                    TaggedSignalReader::I16(Box::new(PcmCodec::from_stream(reader)?))
                }
                KnownSampleType::I32 => {
                    TaggedSignalReader::I32(Box::new(PcmCodec::from_stream(reader)?))
                }
                KnownSampleType::I64 => {
                    TaggedSignalReader::I64(Box::new(PcmCodec::from_stream(reader)?))
                }
                KnownSampleType::U8 => {
                    TaggedSignalReader::U8(Box::new(PcmCodec::from_stream(reader)?))
                }
                KnownSampleType::U16 => {
                    TaggedSignalReader::U16(Box::new(PcmCodec::from_stream(reader)?))
                }
                KnownSampleType::U32 => {
                    TaggedSignalReader::U32(Box::new(PcmCodec::from_stream(reader)?))
                }
                KnownSampleType::U64 => {
                    TaggedSignalReader::U64(Box::new(PcmCodec::from_stream(reader)?))
                }
                KnownSampleType::F32 => {
                    TaggedSignalReader::F32(Box::new(PcmCodec::from_stream(reader)?))
                }
                KnownSampleType::F64 => {
                    TaggedSignalReader::F64(Box::new(PcmCodec::from_stream(reader)?))
                }
            },
        })
    }

    pub fn construct_encoder_writer(
        writer: impl Stream + Write + 'static,
    ) -> Result<TaggedSignalWriter, SyphonError> {
        let spec = writer.spec();
        let sample_type = spec
            .sample_type
            .ok_or(SyphonError::MissingData)?
            .try_into()?;

        Ok(match spec.codec.ok_or(SyphonError::MissingData)? {
            Self::Pcm => match sample_type {
                KnownSampleType::I8 => {
                    TaggedSignalWriter::I8(Box::new(PcmCodec::from_stream(writer)?))
                }
                KnownSampleType::I16 => {
                    TaggedSignalWriter::I16(Box::new(PcmCodec::from_stream(writer)?))
                }
                KnownSampleType::I32 => {
                    TaggedSignalWriter::I32(Box::new(PcmCodec::from_stream(writer)?))
                }
                KnownSampleType::I64 => {
                    TaggedSignalWriter::I64(Box::new(PcmCodec::from_stream(writer)?))
                }
                KnownSampleType::U8 => {
                    TaggedSignalWriter::U8(Box::new(PcmCodec::from_stream(writer)?))
                }
                KnownSampleType::U16 => {
                    TaggedSignalWriter::U16(Box::new(PcmCodec::from_stream(writer)?))
                }
                KnownSampleType::U32 => {
                    TaggedSignalWriter::U32(Box::new(PcmCodec::from_stream(writer)?))
                }
                KnownSampleType::U64 => {
                    TaggedSignalWriter::U64(Box::new(PcmCodec::from_stream(writer)?))
                }
                KnownSampleType::F32 => {
                    TaggedSignalWriter::F32(Box::new(PcmCodec::from_stream(writer)?))
                }
                KnownSampleType::F64 => {
                    TaggedSignalWriter::F64(Box::new(PcmCodec::from_stream(writer)?))
                }
            },
        })
    }

    pub fn construct_encoder_reader<T>(
        &self,
        reader: T,
    ) -> Result<Box<dyn StreamReader>, SyphonError>
    where
        T: SignalReader + 'static,
        T::Sample: FromByteSlice,
    {
        Ok(match self {
            Self::Pcm => Box::new(PcmCodec::from_signal(reader)?),
        })
    }

    pub fn construct_decoder_writer<T>(
        &self,
        writer: T,
    ) -> Result<Box<dyn StreamWriter>, SyphonError>
    where
        T: SignalWriter + 'static,
        T::Sample: FromByteSlice,
    {
        Ok(match self {
            Self::Pcm => Box::new(PcmCodec::from_signal(writer)?),
        })
    }
}

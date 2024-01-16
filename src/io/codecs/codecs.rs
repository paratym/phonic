use crate::{
    io::{
        codecs::pcm::{fill_pcm_stream_spec, PcmCodec},
        KnownSampleType, Stream, StreamReader, StreamSpec, StreamWriter, TaggedSignalReader,
        TaggedSignalWriter,
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

pub trait KnownCodec {
    fn init() -> Result<(), SyphonError>;

    fn fill_spec(&self, spec: &mut StreamSpec) -> Result<(), SyphonError>;

    fn decoder_reader(
        reader: impl Stream<Codec = Self> + Read + 'static,
    ) -> Result<TaggedSignalReader, SyphonError>;

    fn encoder_writer(
        writer: impl Stream<Codec = Self> + Write + 'static,
    ) -> Result<TaggedSignalWriter, SyphonError>;

    fn encoder_reader<T>(
        &self,
        reader: T,
    ) -> Result<Box<dyn StreamReader<Codec = Self>>, SyphonError>
    where
        T: SignalReader + 'static,
        T::Sample: FromByteSlice;

    fn decoder_writer<T>(
        &self,
        writer: T,
    ) -> Result<Box<dyn StreamWriter<Codec = Self>>, SyphonError>
    where
        T: SignalWriter + 'static,
        T::Sample: FromByteSlice;
}

impl KnownCodec for SyphonCodec {
    fn init() -> Result<(), SyphonError> {
        Ok(())
    }

    fn fill_spec(&self, spec: &mut StreamSpec) -> Result<(), SyphonError> {
        match self {
            Self::Pcm => fill_pcm_stream_spec(spec),
        }
    }

    fn decoder_reader(
        reader: impl Stream<Codec = Self> + Read + 'static,
    ) -> Result<TaggedSignalReader, SyphonError> {
        let sample_type = reader
            .spec()
            .sample_type
            .ok_or(SyphonError::MissingData)?
            .try_into()?;

        Ok(match reader.codec().ok_or(SyphonError::MissingData)? {
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

    fn encoder_writer(
        writer: impl Stream<Codec = Self> + Write + 'static,
    ) -> Result<TaggedSignalWriter, SyphonError> {
        let sample_type = writer.spec()
            .sample_type
            .ok_or(SyphonError::MissingData)?
            .try_into()?;

        Ok(match writer.codec().ok_or(SyphonError::MissingData)? {
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

    fn encoder_reader<T>(
        &self,
        reader: T,
    ) -> Result<Box<dyn StreamReader<Codec = Self>>, SyphonError>
    where
        T: SignalReader + 'static,
        T::Sample: FromByteSlice,
    {
        Ok(match self {
            Self::Pcm => Box::new(PcmCodec::from_signal(reader)?),
        })
    }

    fn decoder_writer<T>(
        &self,
        writer: T,
    ) -> Result<Box<dyn StreamWriter<Codec = Self>>, SyphonError>
    where
        T: SignalWriter + 'static,
        T::Sample: FromByteSlice,
    {
        Ok(match self {
            Self::Pcm => Box::new(PcmCodec::from_signal(writer)?),
        })
    }
}

use std::{
    hash::Hash,
    io::{Read, Write},
};
use syphon_core::SyphonError;
use syphon_io_core::{CodecRegistry, CodecTag, Stream, StreamReader, StreamSpec, StreamWriter};
use syphon_signal::{SignalReader, SignalWriter, TaggedSignalReader, TaggedSignalWriter};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum KnownCodec {
    Pcm,
}

impl CodecTag for KnownCodec {}

impl CodecRegistry for KnownCodec {
    fn fill_spec(spec: &mut StreamSpec<Self>) -> Result<(), SyphonError> {
        match spec.codec {
            #[cfg(feature = "pcm")]
            Some(Self::Pcm) => crate::codecs::pcm::fill_pcm_stream_spec(spec),

            _ => Ok(()),
        }
    }

    fn decoder_reader(
        reader: impl Stream<Tag = Self> + 'static,
    ) -> Result<TaggedSignalReader, SyphonError> {
        match reader.spec().codec.ok_or(SyphonError::MissingData)? {
            // Self::Pcm => match sample_type {
            //     KnownSampleType::I8 => {
            //         TaggedSignalReader::I8(Box::new(PcmCodec::from_stream(reader)?))
            //     }
            //     KnownSampleType::I16 => {
            //         TaggedSignalReader::I16(Box::new(PcmCodec::from_stream(reader)?))
            //     }
            //     KnownSampleType::I32 => {
            //         TaggedSignalReader::I32(Box::new(PcmCodec::from_stream(reader)?))
            //     }
            //     KnownSampleType::I64 => {
            //         TaggedSignalReader::I64(Box::new(PcmCodec::from_stream(reader)?))
            //     }
            //     KnownSampleType::U8 => {
            //         TaggedSignalReader::U8(Box::new(PcmCodec::from_stream(reader)?))
            //     }
            //     KnownSampleType::U16 => {
            //         TaggedSignalReader::U16(Box::new(PcmCodec::from_stream(reader)?))
            //     }
            //     KnownSampleType::U32 => {
            //         TaggedSignalReader::U32(Box::new(PcmCodec::from_stream(reader)?))
            //     }
            //     KnownSampleType::U64 => {
            //         TaggedSignalReader::U64(Box::new(PcmCodec::from_stream(reader)?))
            //     }
            //     KnownSampleType::F32 => {
            //         TaggedSignalReader::F32(Box::new(PcmCodec::from_stream(reader)?))
            //     }
            //     KnownSampleType::F64 => {
            //         TaggedSignalReader::F64(Box::new(PcmCodec::from_stream(reader)?))
            //     }
            // },
            _ => Err(SyphonError::Unsupported),
        }
    }

    fn encoder_writer(
        writer: impl Stream<Tag = Self> + 'static,
    ) -> Result<TaggedSignalWriter, SyphonError> {
        match writer.spec().codec.ok_or(SyphonError::MissingData)? {
            // Self::Pcm => match sample_type {
            //     KnownSampleType::I8 => {
            //         TaggedSignalWriter::I8(Box::new(PcmCodec::from_stream(writer)?))
            //     }
            //     KnownSampleType::I16 => {
            //         TaggedSignalWriter::I16(Box::new(PcmCodec::from_stream(writer)?))
            //     }
            //     KnownSampleType::I32 => {
            //         TaggedSignalWriter::I32(Box::new(PcmCodec::from_stream(writer)?))
            //     }
            //     KnownSampleType::I64 => {
            //         TaggedSignalWriter::I64(Box::new(PcmCodec::from_stream(writer)?))
            //     }
            //     KnownSampleType::U8 => {
            //         TaggedSignalWriter::U8(Box::new(PcmCodec::from_stream(writer)?))
            //     }
            //     KnownSampleType::U16 => {
            //         TaggedSignalWriter::U16(Box::new(PcmCodec::from_stream(writer)?))
            //     }
            //     KnownSampleType::U32 => {
            //         TaggedSignalWriter::U32(Box::new(PcmCodec::from_stream(writer)?))
            //     }
            //     KnownSampleType::U64 => {
            //         TaggedSignalWriter::U64(Box::new(PcmCodec::from_stream(writer)?))
            //     }
            //     KnownSampleType::F32 => {
            //         TaggedSignalWriter::F32(Box::new(PcmCodec::from_stream(writer)?))
            //     }
            //     KnownSampleType::F64 => {
            //         TaggedSignalWriter::F64(Box::new(PcmCodec::from_stream(writer)?))
            //     }
            // },
            //
            _ => Err(SyphonError::Unsupported),
        }
    }

    fn encoder_reader(
        &self,
        reader: impl SignalReader + 'static,
    ) -> Result<Box<dyn StreamReader<Tag = Self>>, SyphonError> {
        todo!()
    }

    fn decoder_writer(
        &self,
        writer: impl SignalWriter + 'static,
    ) -> Result<Box<dyn StreamWriter<Tag = Self>>, SyphonError> {
        todo!()
    }
}

#[cfg(feature = "wave")]
impl From<crate::formats::wave::WaveSupportedCodec> for KnownCodec {
    fn from(codec: crate::formats::wave::WaveSupportedCodec) -> Self {
        match codec {
            crate::formats::wave::WaveSupportedCodec::Pcm => Self::Pcm,
        }
    }
}

#[cfg(feature = "wave")]
impl TryFrom<KnownCodec> for crate::formats::wave::WaveSupportedCodec {
    type Error = SyphonError;

    fn try_from(codec: KnownCodec) -> Result<Self, Self::Error> {
        match codec {
            KnownCodec::Pcm => Ok(Self::Pcm),
            _ => Err(SyphonError::Unsupported),
        }
    }
}

#[cfg(feature = "pcm")]
impl From<crate::codecs::pcm::PcmCodecTag> for KnownCodec {
    fn from(_: crate::codecs::pcm::PcmCodecTag) -> Self {
        Self::Pcm
    }
}

#[cfg(feature = "pcm")]
impl TryFrom<KnownCodec> for crate::codecs::pcm::PcmCodecTag {
    type Error = SyphonError;

    fn try_from(codec: KnownCodec) -> Result<Self, Self::Error> {
        match codec {
            KnownCodec::Pcm => Ok(Self()),
            _ => Err(SyphonError::Unsupported),
        }
    }
}

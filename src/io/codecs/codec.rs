use crate::{
    io::{
        codecs::pcm::{fill_pcm_spec, PcmCodec},
        SignalReaderRef, SignalWriterRef, StreamReader, StreamSpecBuilder, StreamWriter,
    },
    SyphonError,
};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum SyphonCodec {
    Pcm,
    Unknown,
}

impl SyphonCodec {
    pub fn fill_spec(&self, spec: &mut StreamSpecBuilder) -> Result<(), SyphonError> {
        match self {
            SyphonCodec::Pcm => fill_pcm_spec(spec),
            SyphonCodec::Unknown => Ok(()),
        }
    }

    pub fn decoder(&self, reader: Box<dyn StreamReader>) -> Result<SignalReaderRef, SyphonError> {
        match self {
            SyphonCodec::Pcm => Ok(PcmCodec::from_stream(reader)?.into()),
            SyphonCodec::Unknown => Err(SyphonError::Unsupported),
        }
    }

    pub fn encoder(&self, writer: Box<dyn StreamWriter>) -> Result<SignalWriterRef, SyphonError> {
        match self {
            SyphonCodec::Pcm => Ok(PcmCodec::from_stream(writer)?.into()),
            SyphonCodec::Unknown => Err(SyphonError::Unsupported),
        }
    }
}

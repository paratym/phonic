use crate::{
    io::{
        codecs::pcm::PcmCodec, StreamReader, StreamWriter, TaggedSignalReader, TaggedSignalWriter,
    },
    SyphonError,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SyphonCodec {
    Pcm,
    Unknown,
}

impl SyphonCodec {
    pub fn construct_decoder(
        &self,
        reader: impl StreamReader + 'static,
    ) -> Result<TaggedSignalReader, SyphonError> {
        match self {
            SyphonCodec::Pcm => {
                TaggedSignalReader::from_dyn_signal(Box::new(PcmCodec::new(reader)?))
            }
            _ => Err(SyphonError::Unsupported),
        }
    }

    pub fn construct_encoder(
        &self,
        writer: impl StreamWriter + 'static,
    ) -> Result<TaggedSignalWriter, SyphonError> {
        match self {
            SyphonCodec::Pcm => {
                TaggedSignalWriter::from_dyn_signal(Box::new(PcmCodec::new(writer)?))
            }
            _ => Err(SyphonError::Unsupported),
        }
    }
}

impl Default for SyphonCodec {
    fn default() -> Self {
        Self::Unknown
    }
}

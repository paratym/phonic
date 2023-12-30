use crate::{
    io::{
        codecs::pcm::{
            construct_pcm_signal_reader_ref, construct_pcm_signal_writer_ref, fill_pcm_spec,
        },
        TaggedSignalReader, TaggedSignalWriter, StreamReader, StreamSpecBuilder, StreamWriter,
    },
    SyphonError,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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

    pub fn construct_decoder(&self, reader: impl StreamReader + 'static) -> Result<TaggedSignalReader, SyphonError> {
        match self {
            SyphonCodec::Pcm => construct_pcm_signal_reader_ref(reader),
            _ => Err(SyphonError::Unsupported),
        }
    }

    pub fn construct_encoder(&self, writer: impl StreamWriter + 'static) -> Result<TaggedSignalWriter, SyphonError> {
        match self {
            SyphonCodec::Pcm => construct_pcm_signal_writer_ref(writer),
            _ => Err(SyphonError::Unsupported),
        }
    }
}

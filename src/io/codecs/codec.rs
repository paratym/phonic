use crate::{
    io::{
        codecs::pcm::{
            construct_pcm_signal_reader_ref, construct_pcm_signal_writer_ref, fill_pcm_spec,
        },
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
        let spec = reader.spec().decoded_spec;

        match self {
            SyphonCodec::Pcm => construct_pcm_signal_reader_ref(reader, spec),
            _ => Err(SyphonError::Unsupported),
        }
    }

    pub fn encoder(&self, writer: Box<dyn StreamWriter>) -> Result<SignalWriterRef, SyphonError> {
        let spec = writer.spec().decoded_spec;

        match self {
            SyphonCodec::Pcm => construct_pcm_signal_writer_ref(writer, spec),
            _ => Err(SyphonError::Unsupported),
        }
    }
}

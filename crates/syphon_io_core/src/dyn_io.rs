use crate::DynStream;
use crate::{CodecTag, DynFormat, FormatTag, TaggedSignal};
use std::io::{Read, Seek, Write};
use syphon_core::SyphonError;

pub trait StdIoStream: Read + Write + Seek {}
impl<T> StdIoStream for T where T: Read + Write + Seek {}

pub trait DynFormatConstructor {
    type Tag: FormatTag;

    fn from_std_io<S: StdIoStream + 'static>(
        &self,
        source: S,
    ) -> Result<Box<dyn DynFormat<Tag = Self::Tag>>, SyphonError>;
}

pub trait DynCodecConstructor {
    type Tag: CodecTag;

    fn from_stream<S: DynStream + 'static>(
        &self,
        stream: S,
    ) -> Result<Box<TaggedSignal>, SyphonError>;

    fn from_signal(
        &self,
        signal: TaggedSignal,
    ) -> Result<Box<dyn DynStream<Tag = Self::Tag>>, SyphonError>;
}

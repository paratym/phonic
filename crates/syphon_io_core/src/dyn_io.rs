use crate::{FormatData, FormatReader, FormatWriter, StreamReader, StreamSpec, StreamWriter};
use std::io::{Read, Write};
use syphon_core::SyphonError;
use syphon_signal::{SignalReader, SignalWriter, TaggedSignalReader, TaggedSignalWriter};

pub trait FormatTag: Sized + Eq + Copy {
    type Codec: CodecTag;
}

pub trait FormatRegistry: FormatTag {
    fn fill_data(data: &mut FormatData<Self>) -> Result<(), SyphonError>;

    fn demux_reader(
        &self,
        inner: impl Read + 'static,
    ) -> Result<Box<dyn FormatReader<Tag = Self>>, SyphonError>;

    fn mux_writer(
        &self,
        inner: impl Write + 'static,
    ) -> Result<Box<dyn FormatWriter<Tag = Self>>, SyphonError>;

    fn mux_reader(
        reader: impl FormatReader<Tag = Self> + 'static,
    ) -> Result<Box<dyn Read>, SyphonError>;

    fn demux_writer(
        writer: impl FormatWriter<Tag = Self> + 'static,
    ) -> Result<Box<dyn Write>, SyphonError>;
}

pub trait CodecTag: Sized + Eq + Copy {}

pub trait CodecRegistry: CodecTag {
    fn fill_spec(spec: &mut StreamSpec<Self>) -> Result<(), SyphonError>;

    fn decoder_reader(
        reader: impl StreamReader<Tag = Self> + 'static,
    ) -> Result<TaggedSignalReader, SyphonError>;

    fn encoder_writer(
        writer: impl StreamWriter<Tag = Self> + 'static,
    ) -> Result<TaggedSignalWriter, SyphonError>;

    fn encoder_reader(
        &self,
        reader: impl SignalReader + 'static,
    ) -> Result<Box<dyn StreamReader<Tag = Self>>, SyphonError>;

    fn decoder_writer(
        &self,
        writer: impl SignalWriter + 'static,
    ) -> Result<Box<dyn StreamWriter<Tag = Self>>, SyphonError>;
}

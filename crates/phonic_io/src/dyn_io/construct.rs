use crate::{
    dyn_io::{DynFormat, DynStream, StdIoSource, TaggedSignal},
    CodecTag, FormatTag, StreamSpec,
};
use phonic_signal::PhonicResult;

pub trait DynFormatConstructor: FormatTag {
    fn read_index<T>(&self, inner: T) -> PhonicResult<Box<dyn DynFormat<Tag = Self>>>
    where
        T: StdIoSource + 'static;

    fn write_index<T, I>(&self, inner: T, index: I) -> PhonicResult<Box<dyn DynFormat<Tag = Self>>>
    where
        T: StdIoSource + 'static,
        I: IntoIterator<Item = StreamSpec<Self::Codec>>;
}

pub trait DynCodecConstructor: CodecTag {
    fn encoder(&self, signal: TaggedSignal) -> PhonicResult<Box<dyn DynStream<Tag = Self>>>;

    fn decoder<S: DynStream<Tag = Self> + 'static>(stream: S) -> PhonicResult<TaggedSignal>;
}

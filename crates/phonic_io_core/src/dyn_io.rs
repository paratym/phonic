use crate::{
    CodecTag, FiniteStream, Format, FormatReader, FormatSeeker, FormatTag, FormatWriter,
    IndexedStream, Stream, StreamReader, StreamSeeker, StreamSpec, StreamWriter, TaggedSignal,
};
use phonic_core::PhonicError;
use phonic_signal::{
    FiniteSignal, IndexedSignal, Signal, SignalReader, SignalSeeker, SignalWriter,
};
use std::io::{Read, Seek, Write};

pub trait StdIoSource: Read + Write + Seek + Send + Sync {}
impl<T> StdIoSource for T where T: Read + Write + Seek + Send + Sync {}

pub trait DynFormat: Format + FormatReader + FormatWriter + FormatSeeker + Send + Sync {}
impl<T> DynFormat for T where T: Format + FormatReader + FormatWriter + FormatSeeker + Send + Sync {}

pub trait DynStream:
    Stream + IndexedStream + FiniteStream + StreamReader + StreamWriter + StreamSeeker + Send + Sync
{
    fn into_decoder(self) -> Result<TaggedSignal, PhonicError>
    where
        Self: Sized + 'static,
        Self::Tag: DynCodecConstructor,
    {
        Self::Tag::decoder(self)
    }
}

impl<T> DynStream for T where
    T: Stream
        + IndexedStream
        + FiniteStream
        + StreamReader
        + StreamWriter
        + StreamSeeker
        + Send
        + Sync
{
}

pub trait DynSignal:
    Signal + IndexedSignal + FiniteSignal + SignalReader + SignalWriter + SignalSeeker + Send + Sync
{
    fn into_encoder<C>(self, codec: C) -> Result<Box<dyn DynStream<Tag = C>>, PhonicError>
    where
        Self: Sized + 'static,
        C: DynCodecConstructor,
        Box<dyn DynSignal<Sample = Self::Sample>>: Into<TaggedSignal>,
    {
        let boxed: Box<dyn DynSignal<Sample = Self::Sample>> = Box::new(self);
        codec.encoder(boxed.into())
    }
}

impl<T> DynSignal for T where
    T: Signal
        + IndexedSignal
        + FiniteSignal
        + SignalReader
        + SignalWriter
        + SignalSeeker
        + Send
        + Sync
{
}

pub trait DynFormatConstructor: FormatTag {
    fn read_index<T>(&self, inner: T) -> Result<Box<dyn DynFormat<Tag = Self>>, PhonicError>
    where
        T: StdIoSource + 'static;

    fn write_index<T, I>(
        &self,
        inner: T,
        index: I,
    ) -> Result<Box<dyn DynFormat<Tag = Self>>, PhonicError>
    where
        T: StdIoSource + 'static,
        I: IntoIterator<Item = StreamSpec<Self::Codec>>;
}

pub trait DynCodecConstructor: CodecTag {
    fn encoder(&self, signal: TaggedSignal) -> Result<Box<dyn DynStream<Tag = Self>>, PhonicError>;

    fn decoder<S: DynStream<Tag = Self> + 'static>(stream: S) -> Result<TaggedSignal, PhonicError>;
}

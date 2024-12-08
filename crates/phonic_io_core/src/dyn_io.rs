use crate::{
    BlockingFormatReader, BlockingFormatWriter, BlockingStreamReader, BlockingStreamWriter,
    CodecTag, FiniteFormat, FiniteStream, Format, FormatReader, FormatSeeker, FormatTag,
    FormatWriter, IndexedFormat, IndexedStream, Stream, StreamReader, StreamSeeker, StreamSpec,
    StreamWriter, TaggedSignal,
};
use phonic_signal::{
    BlockingSignalReader, BlockingSignalWriter, FiniteSignal, IndexedSignal, PhonicResult, Signal,
    SignalReader, SignalSeeker, SignalWriter,
};
use std::io::{Read, Seek, Write};

pub trait StdIoSource: Read + Write + Seek + Send + Sync {}
impl<T> StdIoSource for T where T: Read + Write + Seek + Send + Sync {}

pub trait DynFormat:
    Format
    + IndexedFormat
    + FiniteFormat
    + FormatReader
    + BlockingFormatReader
    + FormatWriter
    + BlockingFormatWriter
    + FormatSeeker
    + Send
    + Sync
{
}

impl<T> DynFormat for T where
    T: Format
        + IndexedFormat
        + FiniteFormat
        + FormatReader
        + BlockingFormatReader
        + FormatWriter
        + BlockingFormatWriter
        + FormatSeeker
        + Send
        + Sync
{
}

pub trait DynStream:
    Stream
    + IndexedStream
    + FiniteStream
    + StreamReader
    + BlockingStreamReader
    + StreamWriter
    + BlockingStreamWriter
    + StreamSeeker
    + Send
    + Sync
{
    fn into_decoder(self) -> PhonicResult<TaggedSignal>
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
        + BlockingStreamReader
        + StreamWriter
        + BlockingStreamWriter
        + StreamSeeker
        + Send
        + Sync
{
}

pub trait DynSignal:
    Signal
    + IndexedSignal
    + FiniteSignal
    + SignalReader
    + BlockingSignalReader
    + SignalWriter
    + BlockingSignalWriter
    + SignalSeeker
    + Send
    + Sync
{
    fn into_encoder<C>(self, codec: C) -> PhonicResult<Box<dyn DynStream<Tag = C>>>
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
        + BlockingSignalReader
        + SignalWriter
        + BlockingSignalWriter
        + SignalSeeker
        + Send
        + Sync
{
}

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

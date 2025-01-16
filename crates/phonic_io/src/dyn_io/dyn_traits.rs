use crate::{
    dyn_io::{DynCodecConstructor, TaggedSignal},
    BlockingFormat, BlockingStream, FiniteFormat, FiniteStream, Format, FormatReader, FormatSeeker,
    FormatWriter, IndexedFormat, IndexedStream, Stream, StreamReader, StreamSeeker, StreamWriter,
};
use phonic_signal::{
    BlockingSignal, FiniteSignal, IndexedSignal, PhonicResult, Signal, SignalReader, SignalSeeker,
    SignalWriter,
};
use std::io::{Read, Seek, Write};

pub trait StdIoSource: Read + Write + Seek + Send + Sync {}
impl<T> StdIoSource for T where T: Read + Write + Seek + Send + Sync {}

pub trait DynFormat:
    Format
    + BlockingFormat
    + IndexedFormat
    + FiniteFormat
    + FormatReader
    + FormatWriter
    + FormatSeeker
    + Send
    + Sync
{
}

impl<T> DynFormat for T where
    T: Format
        + BlockingFormat
        + IndexedFormat
        + FiniteFormat
        + FormatReader
        + FormatWriter
        + FormatSeeker
        + Send
        + Sync
{
}

pub trait DynStream:
    Stream
    + BlockingStream
    + IndexedStream
    + FiniteStream
    + StreamReader
    + StreamWriter
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
        + BlockingStream
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
    Signal
    + BlockingSignal
    + IndexedSignal
    + FiniteSignal
    + SignalReader
    + SignalWriter
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
        + BlockingSignal
        + IndexedSignal
        + FiniteSignal
        + SignalReader
        + SignalWriter
        + SignalSeeker
        + Send
        + Sync
{
}

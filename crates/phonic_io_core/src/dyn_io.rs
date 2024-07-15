use crate::{
    CodecTag, Format, FormatObserver, FormatReader, FormatSeeker, FormatTag, FormatWriter, Stream,
    StreamObserver, StreamReader, StreamSeeker, StreamWriter, TaggedSignal,
};
use phonic_core::PhonicError;
use phonic_signal::{Signal, SignalObserver, SignalReader, SignalSeeker, SignalWriter};
use std::io::{Read, Seek, Write};

pub trait StdIoSource: Read + Write + Seek + Send + Sync {}
impl<T> StdIoSource for T where T: Read + Write + Seek + Send + Sync {}

pub trait DynFormat:
    Format + FormatObserver + FormatReader + FormatWriter + FormatSeeker + Send + Sync
{
}

impl<T> DynFormat for T where
    T: Format + FormatObserver + FormatReader + FormatWriter + FormatSeeker + Send + Sync
{
}

pub trait DynStream:
    Stream + StreamObserver + StreamReader + StreamWriter + StreamSeeker + Send + Sync
{
    fn into_codec(self) -> Result<TaggedSignal, PhonicError>
    where
        Self: Sized + 'static,
        Self::Tag: DynCodecConstructor,
    {
        Self::Tag::from_stream(self)
    }
}

impl<T> DynStream for T where
    T: Stream + StreamObserver + StreamReader + StreamWriter + StreamSeeker + Send + Sync
{
}

pub trait DynSignal:
    Signal + SignalObserver + SignalReader + SignalWriter + SignalSeeker + Send + Sync
{
}
impl<T> DynSignal for T where
    T: Signal + SignalObserver + SignalReader + SignalWriter + SignalSeeker + Send + Sync
{
}

pub trait DynFormatConstructor: FormatTag {
    fn from_std_io<S: StdIoSource + 'static>(
        &self,
        source: S,
    ) -> Result<Box<dyn DynFormat<Tag = Self>>, PhonicError>;

    // fn into_std_io<F: Format + 'static>(format: F) -> Result<Box<dyn StdIoSource>, PhonicError>
    // where
    //     F::Tag: TryInto<Self>;
}

pub trait DynCodecConstructor: CodecTag {
    fn from_stream<S: DynStream<Tag = Self> + 'static>(
        stream: S,
    ) -> Result<TaggedSignal, PhonicError>;

    fn from_signal(
        &self,
        signal: TaggedSignal,
    ) -> Result<Box<dyn DynStream<Tag = Self>>, PhonicError>;
}

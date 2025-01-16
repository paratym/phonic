use crate::{
    utils::{copy_stream_all, copy_stream_exact, DropFinalize, PollIo, StreamSelector},
    BlockingStream, Format, FormatWriter, Stream, StreamDuration, StreamReader, StreamWriter,
};
use phonic_signal::{PhonicError, PhonicResult};
use std::mem::MaybeUninit;

pub trait FormatUtilsExt: Sized + Format {
    fn into_stream(self, stream: usize) -> StreamSelector<Self>
    where
        Self: Sized,
    {
        StreamSelector::new(self, stream)
    }

    fn into_current_stream(self) -> StreamSelector<Self>
    where
        Self: Sized,
    {
        let i = self.current_stream();
        self.into_stream(i)
    }

    fn into_primary_stream(self) -> PhonicResult<StreamSelector<Self>>
    where
        Self: Sized,
    {
        let i = self.primary_stream().ok_or(PhonicError::MissingData)?;
        Ok(self.into_stream(i))
    }

    fn finalize_on_drop(self) -> DropFinalize<Self>
    where
        Self: FormatWriter,
    {
        DropFinalize(self)
    }

    fn polled(self) -> PollIo<Self> {
        PollIo(self)
    }
}

pub trait StreamUtilsExt: Sized + Stream {
    fn copy_exact<R, D>(
        &mut self,
        reader: &mut R,
        duration: D,
        buf: &mut [MaybeUninit<u8>],
    ) -> PhonicResult<()>
    where
        Self: BlockingStream + StreamWriter,
        R: BlockingStream + StreamReader,
        D: StreamDuration,
        Self::Tag: TryInto<R::Tag>,
        PhonicError: From<<Self::Tag as TryInto<R::Tag>>::Error>,
    {
        copy_stream_exact(reader, self, duration, buf)
    }

    // fn copy_exact<R, D>(&mut self, reader: &mut R, duration: D) -> PhonicResult<()>
    // where
    //     Self: BlockingStreamWriter,
    //     R: BlockingStreamReader,
    //     D: StreamDuration,
    //     Self::Tag: TryInto<R::Tag>,
    //     PhonicError: From<<Self::Tag as TryInto<R::Tag>>::Error>,
    // {
    //     // let mut buf = <DefaultBuf<u8>>::new_uninit();
    //     // self.copy_exact_buffered(reader, duration, &mut buf)
    //     todo!()
    // }

    fn copy_all<R>(&mut self, reader: &mut R, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<()>
    where
        Self: BlockingStream + StreamWriter,
        R: BlockingStream + StreamReader,
        Self::Tag: TryInto<R::Tag>,
        PhonicError: From<<Self::Tag as TryInto<R::Tag>>::Error>,
    {
        copy_stream_all(reader, self, buf)
    }

    // fn copy_all<R>(&mut self, reader: &mut R) -> PhonicResult<()>
    // where
    //     Self: BlockingStreamWriter,
    //     R: BlockingStreamReader,
    //     Self::Tag: TryInto<R::Tag>,
    //     PhonicError: From<<Self::Tag as TryInto<R::Tag>>::Error>,
    // {
    //     // let mut buf = <DefaultBuf<u8>>::new_uninit();
    //     // self.copy_all_buffered(reader, &mut buf)
    //     todo!()
    // }

    fn polled(self) -> PollIo<Self> {
        PollIo(self)
    }
}

impl<T> FormatUtilsExt for T where T: Sized + Format {}

impl<T> StreamUtilsExt for T where T: Sized + Stream {}

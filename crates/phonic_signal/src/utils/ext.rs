use crate::{
    utils::{
        copy_all, copy_all_buffered, copy_exact, copy_exact_buffered, Cursor, Indexed, Observer,
        Poll, SignalEvent,
    },
    BlockingSignal, BufferedSignalWriter, DynamicBuf, PhonicResult, ResizeBuf, Signal,
    SignalDuration, SignalReader, SignalWriter, SizedBuf,
};
use std::mem::MaybeUninit;

pub trait SignalUtilsExt: Sized + Signal {
    fn copy_exact<R>(
        &mut self,
        reader: &mut R,
        duration: impl SignalDuration,
        buf: &mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<()>
    where
        Self: BlockingSignal + SignalWriter,
        R: BlockingSignal + SignalReader<Sample = Self::Sample>,
    {
        copy_exact(reader, self, duration, buf)
    }

    fn copy_exact_buffered<R>(
        &mut self,
        reader: &mut R,
        duration: impl SignalDuration,
    ) -> PhonicResult<()>
    where
        Self: BlockingSignal + BufferedSignalWriter,
        R: BlockingSignal + SignalReader<Sample = Self::Sample>,
    {
        copy_exact_buffered(reader, self, duration)
    }

    fn copy_all<R>(
        &mut self,
        reader: &mut R,
        buf: &mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<()>
    where
        Self: BlockingSignal + SignalWriter,
        R: BlockingSignal + SignalReader<Sample = Self::Sample>,
    {
        copy_all(reader, self, buf)
    }

    fn copy_all_buffered<R>(&mut self, reader: &mut R) -> PhonicResult<()>
    where
        Self: BlockingSignal + BufferedSignalWriter,
        R: BlockingSignal + SignalReader<Sample = Self::Sample>,
    {
        copy_all_buffered(reader, self)
    }

    fn take<T>(&mut self) -> PhonicResult<Cursor<T, Self::Sample>>
    where
        Self: SignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        Cursor::read(self)
    }

    fn take_sized<T>(&mut self) -> PhonicResult<Cursor<T, Self::Sample>>
    where
        Self: BlockingSignal + SignalReader,
        T: SizedBuf<Item = Self::Sample>,
    {
        Cursor::read_sized(self)
    }

    fn take_exact<T, D>(&mut self, duration: D) -> PhonicResult<Cursor<T, Self::Sample>>
    where
        Self: BlockingSignal + SignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        D: SignalDuration,
    {
        Cursor::read_exact(self, duration)
    }

    fn take_all<T>(&mut self) -> PhonicResult<Cursor<T, Self::Sample>>
    where
        Self: BlockingSignal + SignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        Cursor::read_all(self)
    }

    fn indexed(self) -> Indexed<Self> {
        Indexed::new(self)
    }

    fn observe<F>(self, callback: F) -> Observer<Self>
    where
        F: for<'s, 'b> Fn(&Self, SignalEvent<'b, Self>) + 'static,
    {
        Observer::new(self, callback)
    }

    fn on_read<F>(self, callback: F) -> Observer<Self>
    where
        F: for<'s, 'b> Fn(&'s Self, &'b [Self::Sample]) + 'static,
    {
        Observer::on_read(self, callback)
    }

    fn on_write<F>(self, callback: F) -> Observer<Self>
    where
        F: for<'s, 'b> Fn(&'s Self, &'b [Self::Sample]) + 'static,
    {
        Observer::on_write(self, callback)
    }

    fn on_seek<F>(self, callback: F) -> Observer<Self>
    where
        F: for<'s> Fn(&'s Self, i64) + 'static,
    {
        Observer::on_seek(self, callback)
    }

    fn polled(self) -> Poll<Self> {
        Poll(self)
    }
}

impl<T: Sized + Signal> SignalUtilsExt for T {}

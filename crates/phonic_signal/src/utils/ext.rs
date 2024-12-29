use crate::{
    utils::{
        copy_all, copy_all_buffered, copy_exact, copy_exact_buffered, Indexed, Observer, Poll,
        SignalEvent,
    },
    BlockingSignalReader, BlockingSignalWriter, BufferedSignalWriter, PhonicResult, Signal,
    SignalDuration,
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
        Self: BlockingSignalWriter,
        R: BlockingSignalReader<Sample = Self::Sample>,
    {
        copy_exact(reader, self, duration, buf)
    }

    fn copy_exact_buffered<R>(
        &mut self,
        reader: &mut R,
        duration: impl SignalDuration,
    ) -> PhonicResult<()>
    where
        Self: BufferedSignalWriter,
        R: BlockingSignalReader<Sample = Self::Sample>,
    {
        copy_exact_buffered(reader, self, duration)
    }

    fn copy_all<R>(
        &mut self,
        reader: &mut R,
        buf: &mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<()>
    where
        Self: BlockingSignalWriter,
        R: BlockingSignalReader<Sample = Self::Sample>,
    {
        copy_all(reader, self, buf)
    }

    fn copy_all_buffered<R>(&mut self, reader: &mut R) -> PhonicResult<()>
    where
        Self: BufferedSignalWriter,
        R: BlockingSignalReader<Sample = Self::Sample>,
    {
        copy_all_buffered(reader, self)
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

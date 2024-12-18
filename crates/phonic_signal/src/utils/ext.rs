use super::{
    copy_all, BufSignal, DefaultBuf, DynamicBuf, Indexed, Observer, Poll, ResizeBuf, SignalEvent,
    SizedBuf, DEFAULT_BUF_LEN,
};
use crate::{utils::copy_exact, BlockingSignalReader, BlockingSignalWriter, PhonicResult, Signal};
use std::{mem::MaybeUninit, time::Duration};

pub trait UtilSignalExt: Sized + Signal {
    fn read_into<T>(&mut self) -> PhonicResult<T>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        T::read(self)
    }

    fn read_into_sized<T>(&mut self) -> PhonicResult<T>
    where
        Self: BlockingSignalReader,
        T: SizedBuf<Item = Self::Sample>,
    {
        T::read(self)
    }

    fn read_into_exact<T>(&mut self, n_frames: usize) -> PhonicResult<T>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
    {
        T::read_exact(self, n_frames)
    }

    fn read_into_exact_interleaved<T>(&mut self, n_samples: usize) -> PhonicResult<T>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
    {
        T::read_exact_interleaved(self, n_samples)
    }

    fn read_into_exact_duration<T>(&mut self, duration: Duration) -> PhonicResult<T>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
    {
        T::read_exact_duration(self, duration)
    }

    fn read_all_into<T>(&mut self) -> PhonicResult<T>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        T::read_all(self)
    }

    fn take<T>(&mut self) -> PhonicResult<BufSignal<T, Self::Sample>>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        BufSignal::read(self)
    }

    fn take_sized<T>(&mut self) -> PhonicResult<BufSignal<T, Self::Sample>>
    where
        Self: BlockingSignalReader,
        T: SizedBuf<Item = Self::Sample>,
    {
        BufSignal::read_sized(self)
    }

    fn take_exact<T>(&mut self, n_frames: usize) -> PhonicResult<BufSignal<T, Self::Sample>>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
    {
        BufSignal::read_exact(self, n_frames)
    }

    fn take_exact_interleaved<T>(
        &mut self,
        n_samples: usize,
    ) -> PhonicResult<BufSignal<T, Self::Sample>>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
    {
        BufSignal::read_exact_interleaved(self, n_samples)
    }

    fn take_exact_duration<T>(
        &mut self,
        duration: Duration,
    ) -> PhonicResult<BufSignal<T, Self::Sample>>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
    {
        BufSignal::read_exact_duration(self, duration)
    }

    fn take_all<T>(&mut self) -> PhonicResult<BufSignal<T, Self::Sample>>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        BufSignal::read_all(self)
    }

    fn copy_exact<R>(&mut self, reader: &mut R, n_frames: u64) -> PhonicResult<()>
    where
        Self: BlockingSignalWriter,
        R: BlockingSignalReader<Sample = Self::Sample>,
    {
        let mut buf = <DefaultBuf<Self::Sample>>::new_uninit();
        copy_exact(reader, self, n_frames, &mut buf)
    }

    fn copy_exact_buffered<R>(
        &mut self,
        reader: &mut R,
        n_frames: u64,
        buf: &mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<()>
    where
        Self: BlockingSignalWriter,
        R: BlockingSignalReader<Sample = Self::Sample>,
    {
        copy_exact(reader, self, n_frames, buf)
    }

    fn copy_all<R>(&mut self, reader: &mut R) -> PhonicResult<()>
    where
        Self: BlockingSignalWriter,
        R: BlockingSignalReader<Sample = Self::Sample>,
    {
        let mut buf = <DefaultBuf<Self::Sample>>::new_uninit();
        copy_all(reader, self, &mut buf)
    }

    fn copy_all_buffered<R>(
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

impl<T: Sized + Signal> UtilSignalExt for T {}

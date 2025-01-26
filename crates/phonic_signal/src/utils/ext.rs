use crate::{
    utils::{
        copy_all, copy_all_buffered, copy_exact, copy_exact_buffered, Cursor, DynamicBuf, Indexed,
        IntoDuration, NFrames, NSamples, Observer, Poll, ResizeBuf, SignalEvent, SizedBuf,
    },
    BlockingSignal, BufferedSignalWriter, FiniteSignal, IndexedSignal, PhonicError, PhonicResult,
    Signal, SignalExt, SignalReader, SignalSeeker, SignalWriter,
};
use std::mem::MaybeUninit;

pub trait SignalUtilsExt: Sized + Signal {
    fn read_into<T>(&mut self) -> PhonicResult<T>
    where
        Self: Sized + SignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        T::read(self)
    }

    fn read_into_sized<T>(&mut self) -> PhonicResult<T>
    where
        Self: Sized + BlockingSignal + SignalReader,
        T: SizedBuf<Item = Self::Sample>,
    {
        T::read(self)
    }

    fn read_into_exact<T>(&mut self, duration: impl IntoDuration<NSamples>) -> PhonicResult<T>
    where
        Self: Sized + BlockingSignal + SignalReader,
        T: DynamicBuf<Item = Self::Sample>,
    {
        T::read_exact(self, duration)
    }

    fn read_all_into<T>(&mut self) -> PhonicResult<T>
    where
        Self: Sized + BlockingSignal + SignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        T::read_all(self)
    }

    fn copy_exact<R>(
        self,
        reader: R,
        duration: impl IntoDuration<NSamples>,
        buf: &mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<()>
    where
        Self: BlockingSignal + SignalWriter,
        R: BlockingSignal + SignalReader<Sample = Self::Sample>,
    {
        copy_exact(reader, self, duration, buf)
    }

    fn copy_exact_buffered<R>(
        self,
        reader: R,
        duration: impl IntoDuration<NSamples>,
    ) -> PhonicResult<()>
    where
        Self: BlockingSignal + BufferedSignalWriter,
        R: BlockingSignal + SignalReader<Sample = Self::Sample>,
    {
        copy_exact_buffered(reader, self, duration)
    }

    fn copy_all<R>(self, reader: R, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<()>
    where
        Self: BlockingSignal + SignalWriter,
        R: BlockingSignal + SignalReader<Sample = Self::Sample>,
    {
        copy_all(reader, self, buf)
    }

    fn copy_all_buffered<R>(self, reader: R) -> PhonicResult<()>
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
        D: IntoDuration<NSamples>,
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

    fn pos_duration<D>(&self) -> D
    where
        Self: IndexedSignal,
        NFrames: IntoDuration<D>,
    {
        NFrames::from(self.pos()).into_duration(self.spec())
    }

    fn len_duration<D>(&self) -> D
    where
        Self: FiniteSignal,
        NFrames: IntoDuration<D>,
    {
        NFrames::from(self.len()).into_duration(self.spec())
    }

    fn rem_duration<D>(&self) -> D
    where
        Self: IndexedSignal + FiniteSignal,
        NFrames: IntoDuration<D>,
    {
        NFrames::from(self.rem()).into_duration(self.spec())
    }

    fn seek_forward<D>(&mut self, duration: D) -> PhonicResult<()>
    where
        Self: SignalSeeker,
        D: IntoDuration<NFrames>,
    {
        let NFrames { n_frames } = duration.into_duration(self.spec());
        self.seek(n_frames as i64)
    }

    fn seek_backward<D>(&mut self, duration: D) -> PhonicResult<()>
    where
        Self: SignalSeeker,
        D: IntoDuration<NFrames>,
    {
        let NFrames { n_frames } = duration.into_duration(self.spec());
        self.seek(-(n_frames as i64))
    }

    fn seek_from_start<D>(&mut self, duration: D) -> PhonicResult<()>
    where
        Self: IndexedSignal + SignalSeeker,
        D: IntoDuration<NFrames>,
    {
        let NFrames { n_frames: pos } = self.pos_duration();
        let NFrames { n_frames: new_pos } = duration.into_duration(self.spec());

        let offset = if new_pos >= pos {
            (new_pos - pos) as i64
        } else {
            -((pos - new_pos) as i64)
        };

        self.seek(offset)
    }

    fn seek_from_end<D>(&mut self, duration: D) -> PhonicResult<()>
    where
        Self: IndexedSignal + FiniteSignal + SignalSeeker,
        D: IntoDuration<NFrames>,
    {
        let NFrames { n_frames } = duration.into_duration(self.spec());
        let new_pos: NFrames = self
            .len()
            .checked_sub(n_frames)
            .ok_or(PhonicError::out_of_bounds())?
            .into();

        self.seek_from_start(new_pos)
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

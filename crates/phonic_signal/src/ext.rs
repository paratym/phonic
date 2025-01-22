use crate::{
    block_on_signal, utils::slice_as_init_mut, BlockingSignal, BufferedSignalReader, FiniteSignal,
    IndexedSignal, PhonicError, PhonicResult, Signal, SignalReader, SignalSeeker, SignalWriter,
};
use std::mem::MaybeUninit;

pub trait SignalExt: Signal {
    fn is_empty(&self) -> bool
    where
        Self: FiniteSignal,
    {
        self.len() == 0
    }

    fn is_exhausted(&self) -> bool
    where
        Self: IndexedSignal + FiniteSignal,
    {
        self.pos() == self.len()
    }

    fn rem(&self) -> u64
    where
        Self: IndexedSignal + FiniteSignal,
    {
        self.len() - self.pos()
    }

    fn read_init<'a>(
        &mut self,
        buf: &'a mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<&'a mut [Self::Sample]>
    where
        Self: SignalReader,
    {
        let n_samples = self.read(buf)?;
        let uninit_slice = &mut buf[..n_samples];
        let init_slice = unsafe { slice_as_init_mut(uninit_slice) };

        Ok(init_slice)
    }

    fn read_blocking(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize>
    where
        Self: BlockingSignal + SignalReader,
    {
        block_on_signal!(self, self.read(buf))
    }

    fn read_init_blocking<'a>(
        &mut self,
        buf: &'a mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<&'a mut [Self::Sample]>
    where
        Self: BlockingSignal + SignalReader,
    {
        // pointer hack to avoid "mutably borrowed on previous iteration of loop"
        block_on_signal!(self, self.read_init(buf), result => result.map(|init| unsafe {
            std::slice::from_raw_parts_mut(init.as_mut_ptr(), init.len())
        }))
    }

    fn read_exact(&mut self, mut buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<()>
    where
        Self: BlockingSignal + SignalReader,
    {
        if buf.len() % self.spec().channels.count() as usize != 0 {
            return Err(PhonicError::InvalidInput);
        }

        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => buf = &mut buf[n..],
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => self.block(),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    fn read_exact_init<'a>(
        &mut self,
        buf: &'a mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<&'a mut [Self::Sample]>
    where
        Self: BlockingSignal + SignalReader,
    {
        self.read_exact(buf)?;
        let init_slice = unsafe { slice_as_init_mut(buf) };

        Ok(init_slice)
    }

    fn fill_blocking(&mut self) -> PhonicResult<&[Self::Sample]>
    where
        Self: BlockingSignal + BufferedSignalReader,
    {
        // pointer hack to avoid "mutably borrowed on previous iteration of loop"
        block_on_signal!(self, self.fill(), result => result.map(|init| unsafe {
            std::slice::from_raw_parts(init.as_ptr(), init.len())
        }))
    }

    fn read_frames<'a>(
        &mut self,
        buf: &'a mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<impl Iterator<Item = &'a [Self::Sample]>>
    where
        Self: SignalReader,
    {
        let samples = self.read_init(buf)?;
        let n_channels = self.spec().channels.count() as usize;
        debug_assert_eq!(samples.len() % n_channels, 0);

        Ok(samples.chunks_exact(n_channels))
    }

    fn read_exact_frames<'a>(
        &mut self,
        buf: &'a mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<impl Iterator<Item = &'a [Self::Sample]>>
    where
        Self: BlockingSignal + SignalReader,
    {
        let samples = self.read_exact_init(buf)?;
        let n_channels = self.spec().channels.count() as usize;
        debug_assert_eq!(samples.len() % n_channels, 0);

        Ok(samples.chunks_exact(n_channels))
    }

    fn write_blocking(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize>
    where
        Self: BlockingSignal + SignalWriter,
    {
        block_on_signal!(self, self.write(buf))
    }

    fn flush_blocking(&mut self) -> PhonicResult<()>
    where
        Self: BlockingSignal + SignalWriter,
    {
        block_on_signal!(self, self.flush())
    }

    fn write_exact(&mut self, mut buf: &[Self::Sample]) -> PhonicResult<()>
    where
        Self: BlockingSignal + SignalWriter,
    {
        if buf.len() % self.spec().channels.count() as usize != 0 {
            return Err(PhonicError::InvalidInput);
        }

        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => buf = &buf[n..],
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => self.block(),
                Err(e) => return Err(e),
            };
        }

        Ok(())
    }

    fn seek_to_start(&mut self) -> PhonicResult<()>
    where
        Self: IndexedSignal + SignalSeeker,
    {
        self.seek(-(self.pos() as i64))
    }

    fn seek_to_end(&mut self) -> crate::PhonicResult<()>
    where
        Self: IndexedSignal + FiniteSignal + SignalSeeker,
    {
        self.seek((self.len() - self.pos()) as i64)
    }
}

impl<T: Signal> SignalExt for T {}

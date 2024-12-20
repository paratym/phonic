use crate::{
    utils::{copy_to_uninit_slice, slice_as_uninit_mut, DynamicBuf, OwnedBuf, ResizeBuf, SizedBuf},
    BlockingSignalReader, BufferedSignal, BufferedSignalReader, BufferedSignalWriter, FiniteSignal,
    IndexedSignal, IntoDuration, NFrames, NSamples, PhonicError, PhonicResult, Sample, Signal,
    SignalDuration, SignalReader, SignalSeeker, SignalSpec, SignalWriter,
};
use std::{
    borrow::{Borrow, BorrowMut},
    marker::PhantomData,
    mem::MaybeUninit,
};

pub struct BufSignal<B, S> {
    spec: SignalSpec,
    buf: B,
    i: usize,
    _sample: PhantomData<S>,
}

impl<B, S> BufSignal<B, S> {
    pub fn new(spec: SignalSpec, buf: B) -> Self {
        Self {
            spec,
            buf,
            i: 0,
            _sample: PhantomData,
        }
    }

    pub fn new_uninit<D>(spec: SignalSpec, duration: D) -> BufSignal<B::Uninit, MaybeUninit<S>>
    where
        B: DynamicBuf,
        D: SignalDuration,
    {
        let NSamples { n_samples } = duration.into_duration(&spec);
        debug_assert_eq!(n_samples % spec.channels.count() as u64, 0);

        let buf = B::new_uninit(n_samples as usize);
        BufSignal::new(spec, buf)
    }

    pub fn new_uninit_sized(spec: SignalSpec) -> BufSignal<B::Uninit, MaybeUninit<S>>
    where
        B: SizedBuf,
    {
        let buf = B::new_uninit();
        debug_assert_eq!(buf._as_slice().len() % spec.channels.count() as usize, 0);

        BufSignal::new(spec, buf)
    }

    pub fn silence<D>(spec: SignalSpec, duration: D) -> Self
    where
        B: DynamicBuf,
        B::Item: Sample,
        D: SignalDuration,
    {
        let NSamples { n_samples } = duration.into_duration(&spec);
        debug_assert_eq!(n_samples % spec.channels.count() as u64, 0);

        let buf = B::silence(n_samples as usize);
        Self::new(spec, buf)
    }

    pub fn silence_sized(spec: SignalSpec) -> Self
    where
        B: SizedBuf,
        B::Item: Sample,
    {
        let buf = B::silence();
        debug_assert_eq!(buf._as_slice().len() % spec.channels.count() as usize, 0);

        Self::new(spec, buf)
    }

    pub fn read<R>(reader: &mut R) -> PhonicResult<Self>
    where
        B: DynamicBuf,
        B::Uninit: ResizeBuf,
        R: BlockingSignalReader<Sample = B::Item>,
    {
        let spec = *reader.spec();
        let buf = B::read(reader)?;
        debug_assert_eq!(buf._as_slice().len() % spec.channels.count() as usize, 0);

        Ok(Self::new(spec, buf))
    }

    pub fn read_sized<R>(reader: &mut R) -> PhonicResult<Self>
    where
        B: SizedBuf,
        R: BlockingSignalReader<Sample = B::Item>,
    {
        let spec = *reader.spec();
        let buf = B::read(reader)?;
        debug_assert_eq!(buf._as_slice().len() % spec.channels.count() as usize, 0);

        Ok(Self::new(spec, buf))
    }

    pub fn read_exact<R, D>(reader: &mut R, duration: D) -> PhonicResult<Self>
    where
        B: DynamicBuf,
        R: BlockingSignalReader<Sample = B::Item>,
        D: SignalDuration,
    {
        let spec = *reader.spec();
        let buf = B::read_exact(reader, duration)?;

        Ok(Self::new(spec, buf))
    }

    pub fn read_all<R>(reader: &mut R) -> PhonicResult<Self>
    where
        B: DynamicBuf,
        B::Uninit: ResizeBuf,
        R: BlockingSignalReader<Sample = B::Item>,
    {
        let spec = *reader.spec();
        let buf = B::read_all(reader)?;

        Ok(Self::new(spec, buf))
    }
}

impl<B, S: Sample> Signal for BufSignal<B, S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<B, S: Sample> Signal for BufSignal<B, MaybeUninit<S>> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<B, S> BufSignal<B, S> {
    fn _commit_samples(&mut self, n_samples: usize) {
        debug_assert_eq!(n_samples % self.spec.channels.count() as usize, 0);
        self.i += n_samples
    }
}

impl<B, S: Sample> BufferedSignal for BufSignal<B, S> {
    fn commit_samples(&mut self, n_samples: usize) {
        self._commit_samples(n_samples);
    }
}

impl<B, S: Sample> BufferedSignal for BufSignal<B, MaybeUninit<S>> {
    fn commit_samples(&mut self, n_samples: usize) {
        self._commit_samples(n_samples);
    }
}

impl<B, S> BufSignal<B, S> {
    fn _pos(&self) -> u64 {
        let NFrames { n_frames } = NSamples::from(self.i as u64).into_duration(&self.spec);

        n_frames
    }
}

impl<B, S: Sample> IndexedSignal for BufSignal<B, S> {
    fn pos(&self) -> u64 {
        self._pos()
    }
}

impl<B, S: Sample> IndexedSignal for BufSignal<B, MaybeUninit<S>> {
    fn pos(&self) -> u64 {
        self._pos()
    }
}

impl<B, S> BufSignal<B, S> {
    fn _len(&self, len: usize) -> u64 {
        let NFrames { n_frames } = NSamples::from(len as u64).into_duration(&self.spec);

        n_frames
    }
}

impl<B, S> FiniteSignal for BufSignal<B, S>
where
    B: Borrow<[S]>,
    S: Sample,
{
    fn len(&self) -> u64 {
        let buf_len = self.buf.borrow().len();
        self._len(buf_len)
    }
}

impl<B, S> FiniteSignal for BufSignal<B, MaybeUninit<S>>
where
    B: Borrow<[MaybeUninit<S>]>,
    S: Sample,
{
    fn len(&self) -> u64 {
        let buf_len = self.buf.borrow().len();
        self._len(buf_len)
    }
}

impl<B, S> SignalReader for BufSignal<B, S>
where
    B: Borrow<[S]>,
    S: Sample,
{
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let n_samples = buf.len().min(self.buf.borrow().len() - self.i);
        let inner_slice = &self.buf.borrow()[self.i..self.i + n_samples];
        copy_to_uninit_slice(inner_slice, &mut buf[..n_samples]);

        self.i += n_samples;
        Ok(n_samples)
    }
}

impl<B, S> BufferedSignalReader for BufSignal<B, S>
where
    B: Borrow<[S]>,
    S: Sample,
{
    fn available_samples(&self) -> &[Self::Sample] {
        &self.buf.borrow()[self.i..]
    }
}

impl<B, S> SignalWriter for BufSignal<B, S>
where
    B: BorrowMut<[S]>,
    S: Sample,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let inner_buf = self.buf.borrow_mut();
        let n_samples = buf.len().min(inner_buf.len() - self.i);
        let inner_slice = &mut inner_buf[self.i..self.i + n_samples];
        inner_slice.copy_from_slice(&buf[..n_samples]);

        self.i += n_samples;
        Ok(n_samples)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Ok(())
    }
}

impl<B, S> SignalWriter for BufSignal<B, MaybeUninit<S>>
where
    B: BorrowMut<[MaybeUninit<S>]>,
    S: Sample,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let inner_buf = self.buf.borrow_mut();
        let n_samples = buf.len().min(inner_buf.len() - self.i);
        let inner_slice = &mut inner_buf[self.i..self.i + n_samples];
        copy_to_uninit_slice(&buf[..n_samples], inner_slice);

        self.i += n_samples;
        Ok(n_samples)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Ok(())
    }
}

impl<B, S> BufferedSignalWriter for BufSignal<B, S>
where
    B: BorrowMut<[S]>,
    S: Sample,
{
    fn available_slots(&mut self) -> &mut [MaybeUninit<Self::Sample>] {
        let init_buf = &mut self.buf.borrow_mut()[self.i..];
        slice_as_uninit_mut(init_buf)
    }
}

impl<B, S> BufferedSignalWriter for BufSignal<B, MaybeUninit<S>>
where
    B: BorrowMut<[MaybeUninit<S>]>,
    S: Sample,
{
    fn available_slots(&mut self) -> &mut [MaybeUninit<Self::Sample>] {
        &mut self.buf.borrow_mut()[self.i..]
    }
}

impl<B, S> BufSignal<B, S>
where
    Self: IndexedSignal + FiniteSignal,
{
    fn _seek(&mut self, offset: i64) -> PhonicResult<()> {
        let new_pos = self
            .pos()
            .checked_add_signed(offset)
            .ok_or(PhonicError::OutOfBounds)?;

        if new_pos > self.len() {
            return Err(PhonicError::OutOfBounds);
        }

        let NSamples { n_samples } = NFrames::from(new_pos).into_duration(self.spec());
        self.i = n_samples as usize;

        Ok(())
    }
}

impl<B, S> SignalSeeker for BufSignal<B, S>
where
    B: Borrow<[S]>,
    S: Sample,
{
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        self._seek(offset)
    }
}

impl<B, S> SignalSeeker for BufSignal<B, MaybeUninit<S>>
where
    B: Borrow<[MaybeUninit<S>]>,
    S: Sample,
{
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        self._seek(offset)
    }
}

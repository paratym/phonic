use crate::{
    utils::{copy_to_uninit_slice, slice_as_uninit_mut},
    BlockingSignal, BufferedSignalReader, BufferedSignalWriter, DynamicBuf, FiniteSignal,
    IndexedSignal, IntoDuration, NFrames, NSamples, OwnedBuf, PhonicError, PhonicResult, ResizeBuf,
    Sample, Signal, SignalDuration, SignalExt, SignalReader, SignalSeeker, SignalSpec,
    SignalWriter, SizedBuf,
};
use std::{
    borrow::{Borrow, BorrowMut},
    marker::PhantomData,
    mem::MaybeUninit,
};

pub struct Cursor<B, S> {
    spec: SignalSpec,
    buf: B,
    i: usize,
    _sample: PhantomData<S>,
}

impl<B, S> Cursor<B, S> {
    pub fn new(spec: SignalSpec, buf: B) -> Self {
        Self {
            spec,
            buf,
            i: 0,
            _sample: PhantomData,
        }
    }

    pub fn uninit<D>(spec: SignalSpec, duration: D) -> Cursor<B::Uninit, MaybeUninit<S>>
    where
        B: DynamicBuf,
        D: SignalDuration,
    {
        let NSamples { n_samples } = duration.into_duration(&spec);
        debug_assert_eq!(n_samples % spec.channels.count() as u64, 0);

        let buf = B::uninit(n_samples as usize);
        Cursor::new(spec, buf)
    }

    pub fn uninit_sized(spec: SignalSpec) -> Cursor<B::Uninit, MaybeUninit<S>>
    where
        B: SizedBuf,
    {
        let buf = B::uninit();
        debug_assert_eq!(buf._as_slice().len() % spec.channels.count() as usize, 0);

        Cursor::new(spec, buf)
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
        R: SignalReader<Sample = B::Item>,
    {
        let spec = *reader.spec();
        let buf: B = reader.read_into()?;
        debug_assert_eq!(buf._as_slice().len() % spec.channels.count() as usize, 0);

        Ok(Self::new(spec, buf))
    }

    pub fn read_sized<R>(reader: &mut R) -> PhonicResult<Self>
    where
        B: SizedBuf,
        R: BlockingSignal + SignalReader<Sample = B::Item>,
    {
        let spec = *reader.spec();
        let buf: B = reader.read_into_sized()?;
        debug_assert_eq!(buf._as_slice().len() % spec.channels.count() as usize, 0);

        Ok(Self::new(spec, buf))
    }

    pub fn read_exact<R, D>(reader: &mut R, duration: D) -> PhonicResult<Self>
    where
        B: DynamicBuf,
        R: BlockingSignal + SignalReader<Sample = B::Item>,
        D: SignalDuration,
    {
        let spec = *reader.spec();
        let buf = reader.read_into_exact(duration)?;

        Ok(Self::new(spec, buf))
    }

    pub fn read_all<R>(reader: &mut R) -> PhonicResult<Self>
    where
        B: DynamicBuf,
        B::Uninit: ResizeBuf,
        R: BlockingSignal + SignalReader<Sample = B::Item>,
    {
        let spec = *reader.spec();
        let buf = reader.read_all_into()?;

        Ok(Self::new(spec, buf))
    }

    fn _pos(&self) -> u64 {
        let NFrames { n_frames } = NSamples::from(self.i as u64).into_duration(&self.spec);
        n_frames
    }

    fn _len(&self, len: usize) -> u64 {
        let NFrames { n_frames } = NSamples::from(len as u64).into_duration(&self.spec);
        n_frames
    }

    fn _commit(&mut self, n_samples: usize, len: usize) {
        assert!(n_samples <= len - self.i);
        self.i += n_samples;
    }

    fn _seek(&mut self, offset: i64, len: usize) -> PhonicResult<()> {
        let new_pos = self
            ._pos()
            .checked_add_signed(offset)
            .ok_or(PhonicError::OutOfBounds)?;

        if new_pos > len as u64 {
            return Err(PhonicError::OutOfBounds);
        }

        let NSamples { n_samples } = NFrames::from(new_pos).into_duration(&self.spec);
        self.i = n_samples as usize;

        Ok(())
    }
}

impl<B, S: Sample> Signal for Cursor<B, S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<B, S: Sample> Signal for Cursor<B, MaybeUninit<S>> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<B, S: Sample> IndexedSignal for Cursor<B, S> {
    fn pos(&self) -> u64 {
        self._pos()
    }
}

impl<B, S: Sample> IndexedSignal for Cursor<B, MaybeUninit<S>> {
    fn pos(&self) -> u64 {
        self._pos()
    }
}

impl<B, S> FiniteSignal for Cursor<B, S>
where
    B: Borrow<[S]>,
    S: Sample,
{
    fn len(&self) -> u64 {
        let buf_len = self.buf.borrow().len();
        self._len(buf_len)
    }
}

impl<B, S> FiniteSignal for Cursor<B, MaybeUninit<S>>
where
    B: Borrow<[MaybeUninit<S>]>,
    S: Sample,
{
    fn len(&self) -> u64 {
        let buf_len = self.buf.borrow().len();
        self._len(buf_len)
    }
}

impl<B, S> SignalReader for Cursor<B, S>
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

impl<B, S> BufferedSignalReader for Cursor<B, S>
where
    B: Borrow<[S]>,
    S: Sample,
{
    fn fill(&mut self) -> crate::PhonicResult<&[Self::Sample]> {
        Ok(&self.buf.borrow()[self.i..])
    }

    fn buffer(&self) -> Option<&[Self::Sample]> {
        match self.buf.borrow()[self.i..] {
            [] => None,
            ref buf => Some(buf),
        }
    }

    fn consume(&mut self, n_samples: usize) {
        self._commit(n_samples, self.buf.borrow().len())
    }
}

impl<B, S> SignalWriter for Cursor<B, S>
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

impl<B, S> SignalWriter for Cursor<B, MaybeUninit<S>>
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

impl<B, S> BufferedSignalWriter for Cursor<B, S>
where
    B: BorrowMut<[S]>,
    S: Sample,
{
    fn buffer_mut(&mut self) -> Option<&mut [MaybeUninit<Self::Sample>]> {
        match self.buf.borrow_mut()[self.i..] {
            [] => None,
            ref mut buf => Some(slice_as_uninit_mut(buf)),
        }
    }

    fn commit(&mut self, n_samples: usize) {
        self._commit(n_samples, self.buf.borrow().len())
    }
}

impl<B, S> BufferedSignalWriter for Cursor<B, MaybeUninit<S>>
where
    B: BorrowMut<[MaybeUninit<S>]>,
    S: Sample,
{
    fn buffer_mut(&mut self) -> Option<&mut [MaybeUninit<Self::Sample>]> {
        match self.buf.borrow_mut()[self.i..] {
            [] => None,
            ref mut buf => Some(buf),
        }
    }

    fn commit(&mut self, n_samples: usize) {
        self._commit(n_samples, self.buf.borrow().len())
    }
}

impl<B, S> SignalSeeker for Cursor<B, S>
where
    B: Borrow<[S]>,
    S: Sample,
{
    fn seek(&mut self, n_frames: i64) -> PhonicResult<()> {
        let len = self.buf.borrow().len();
        self._seek(n_frames, len)
    }
}

impl<B, S> SignalSeeker for Cursor<B, MaybeUninit<S>>
where
    B: Borrow<[MaybeUninit<S>]>,
    S: Sample,
{
    fn seek(&mut self, n_frames: i64) -> PhonicResult<()> {
        let len = self.buf.borrow().len();
        self._seek(n_frames, len)
    }
}

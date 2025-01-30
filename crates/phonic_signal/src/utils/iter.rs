use crate::{
    utils::{copy_to_uninit_slice, IntoDuration, NFrames, NSamples},
    FiniteSignal, PhonicResult, Sample, Signal, SignalReader, SignalSpec, SignalWriter,
};
use std::{
    borrow::{Borrow, BorrowMut},
    marker::PhantomData,
    mem::MaybeUninit,
};

pub struct SampleIterSignal<T, S> {
    iter: T,
    spec: SignalSpec,
    _sample: PhantomData<S>,
}

pub struct FrameIterSignal<T, S> {
    iter: T,
    spec: SignalSpec,
    _sample: PhantomData<S>,
}

impl<T, S> SampleIterSignal<T, S> {
    pub fn new(iter: T, spec: SignalSpec) -> Self {
        Self {
            iter,
            spec,
            _sample: PhantomData,
        }
    }
}

impl<T, S> FrameIterSignal<T, S> {
    pub fn new(iter: T, spec: SignalSpec) -> Self {
        Self {
            iter,
            spec,
            _sample: PhantomData,
        }
    }
}

impl<T, S: Sample> Signal for SampleIterSignal<T, S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T, S: Sample> Signal for SampleIterSignal<T, MaybeUninit<S>> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T, S> FiniteSignal for SampleIterSignal<T, S>
where
    T: ExactSizeIterator,
    S: Sample,
{
    fn len(&self) -> u64 {
        let NFrames { n_frames } =
            NSamples::from(self.iter.len() as u64).into_duration(self.spec());

        n_frames
    }
}

impl<T, S> FiniteSignal for SampleIterSignal<T, MaybeUninit<S>>
where
    T: ExactSizeIterator,
    S: Sample,
{
    fn len(&self) -> u64 {
        let NFrames { n_frames } =
            NSamples::from(self.iter.len() as u64).into_duration(self.spec());

        n_frames
    }
}

impl<T, S> SignalReader for SampleIterSignal<T, S>
where
    T: Iterator,
    T::Item: Borrow<S>,
    S: Sample,
{
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let mut buf_len = buf.len();
        let n_channels = self.spec().n_channels;
        buf_len -= buf_len % n_channels;

        let mut i = 0;
        while i < buf_len {
            let Some(sample) = self.iter.next() else {
                break;
            };

            buf[i].write(*sample.borrow());
            i += 1;
        }

        debug_assert_eq!(i % n_channels, 0);
        Ok(i)
    }
}

impl<T, S> SignalWriter for SampleIterSignal<T, S>
where
    T: Iterator,
    T::Item: BorrowMut<S>,
    S: Sample,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let mut buf_len = buf.len();
        let n_channels = self.spec().n_channels;
        buf_len -= buf_len % n_channels;

        let mut i = 0;
        while i < buf_len {
            let Some(mut sample) = self.iter.next() else {
                break;
            };

            *sample.borrow_mut() = buf[i];
            i += 1;
        }

        debug_assert_eq!(i % n_channels, 0);
        Ok(i)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Ok(())
    }
}

impl<T, S> SignalWriter for SampleIterSignal<T, MaybeUninit<S>>
where
    T: Iterator,
    T::Item: BorrowMut<MaybeUninit<S>>,
    S: Sample,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let mut buf_len = buf.len();
        let n_channels = self.spec.n_channels;
        buf_len -= buf_len % n_channels;

        let mut i = 0;
        while i < buf_len {
            let Some(mut sample) = self.iter.next() else {
                break;
            };

            sample.borrow_mut().write(buf[i]);
            i += 1;
        }

        debug_assert_eq!(i % n_channels, 0);
        Ok(i)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Ok(())
    }
}

impl<T, S: Sample> Signal for FrameIterSignal<T, S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T, S: Sample> Signal for FrameIterSignal<T, MaybeUninit<S>> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T, S> FiniteSignal for FrameIterSignal<T, S>
where
    T: ExactSizeIterator,
    S: Sample,
{
    fn len(&self) -> u64 {
        self.iter.len() as u64
    }
}

impl<T, S> FiniteSignal for FrameIterSignal<T, MaybeUninit<S>>
where
    T: ExactSizeIterator,
    S: Sample,
{
    fn len(&self) -> u64 {
        self.iter.len() as u64
    }
}

impl<T, S> SignalReader for FrameIterSignal<T, S>
where
    T: Iterator,
    T::Item: Borrow<[S]>,
    S: Sample,
{
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let mut buf_len = buf.len();
        let n_channels = self.spec.n_channels;
        buf_len -= buf_len % n_channels;

        let mut i = 0;
        while i < buf_len {
            let Some(frame) = self.iter.next() else {
                break;
            };

            copy_to_uninit_slice(frame.borrow(), &mut buf[i..i + n_channels]);

            i += n_channels;
        }

        Ok(i)
    }
}

impl<T, S> SignalWriter for FrameIterSignal<T, S>
where
    T: Iterator,
    T::Item: BorrowMut<[S]>,
    S: Sample,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let mut buf_len = buf.len();
        let n_channels = self.spec.n_channels;
        buf_len -= buf_len % n_channels;

        let mut i = 0;
        while i < buf_len {
            let Some(mut frame) = self.iter.next() else {
                break;
            };

            frame.borrow_mut().copy_from_slice(&buf[i..i + n_channels]);
            i += n_channels;
        }

        Ok(i)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Ok(())
    }
}

impl<T, S> SignalWriter for FrameIterSignal<T, MaybeUninit<S>>
where
    T: Iterator,
    T::Item: BorrowMut<[MaybeUninit<S>]>,
    S: Sample,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let mut buf_len = buf.len();
        let n_channels = self.spec.n_channels;
        buf_len -= buf_len % n_channels;

        let mut i = 0;
        while i < buf_len {
            let Some(mut frame) = self.iter.next() else {
                break;
            };

            copy_to_uninit_slice(&buf[i..i + n_channels], frame.borrow_mut());
            i += n_channels;
        }

        Ok(i)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Ok(())
    }
}

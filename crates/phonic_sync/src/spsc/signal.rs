use crate::spsc::{Consumer, Producer, SpscBuf};
use phonic_signal::{
    utils::{
        copy_to_uninit_slice, DefaultDynamicBuf, DefaultSizedBuf, DynamicBuf, IntoDuration,
        NSamples, OwnedBuf, SizedBuf,
    },
    BufferedSignalReader, BufferedSignalWriter, PhonicError, PhonicResult, Sample, Signal,
    SignalReader, SignalSpec, SignalWriter,
};
use std::{mem::MaybeUninit, sync::atomic::Ordering};

pub struct SpscSignal;

pub struct SignalProducer<T, B> {
    spec: SignalSpec,
    producer: Producer<T, B>,
}

pub struct SignalConsumer<T, B> {
    spec: SignalSpec,
    consumer: Consumer<T, B>,
}

type SpscSignalPair<T, B> = (SignalProducer<T, B>, SignalConsumer<T, B>);

impl SpscSignal {
    pub unsafe fn from_raw_parts<T, B>(
        spec: SignalSpec,
        buf: B,
        ptr: *mut MaybeUninit<T>,
        cap: usize,
    ) -> SpscSignalPair<T, B> {
        let aligned_cap = cap - cap % spec.n_channels;
        let (producer, consumer) = SpscBuf::from_raw_parts(buf, ptr, aligned_cap);

        (
            SignalProducer { spec, producer },
            SignalConsumer { spec, consumer },
        )
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new<T, B>(spec: SignalSpec, mut buf: B) -> SpscSignalPair<T, B>
    where
        B: AsMut<[T]>,
    {
        let slice = buf.as_mut();
        let ptr = slice.as_mut_ptr().cast();
        let cap = slice.len();

        unsafe { Self::from_raw_parts(spec, buf, ptr, cap) }
    }

    pub fn new_uninit<T, B>(spec: SignalSpec, mut buf: B) -> SpscSignalPair<T, B>
    where
        B: AsMut<[MaybeUninit<T>]>,
    {
        let slice = buf.as_mut();
        let ptr = slice.as_mut_ptr();
        let cap = slice.len();

        unsafe { Self::from_raw_parts(spec, buf, ptr, cap) }
    }

    pub fn new_sized<B>(spec: SignalSpec) -> SpscSignalPair<B::Item, B::Uninit>
    where
        B: SizedBuf,
        B::Uninit: AsMut<[<B::Uninit as OwnedBuf>::Item]>,
    {
        let buf = B::uninit();
        Self::new_uninit(spec, buf)
    }

    pub fn default_sized<T>(
        spec: SignalSpec,
    ) -> SpscSignalPair<T, <DefaultSizedBuf<T> as OwnedBuf>::Uninit> {
        let buf = DefaultSizedBuf::<T>::uninit();
        Self::new_uninit(spec, buf)
    }

    pub fn new_duration<B>(
        spec: SignalSpec,
        duration: impl IntoDuration<NSamples>,
    ) -> SpscSignalPair<B::Item, B::Uninit>
    where
        B: DynamicBuf,
        B::Uninit: AsMut<[MaybeUninit<B::Item>]>,
    {
        let NSamples { n_samples } = duration.into_duration(&spec);
        let buf = B::uninit(n_samples as usize);
        Self::new_uninit(spec, buf)
    }

    pub fn default_duration<T>(
        spec: SignalSpec,
        duration: impl IntoDuration<NSamples>,
    ) -> SpscSignalPair<T, <DefaultDynamicBuf<T> as OwnedBuf>::Uninit> {
        let NSamples { n_samples } = duration.into_duration(&spec);
        let buf = DefaultDynamicBuf::<T>::uninit(n_samples as usize);
        Self::new_uninit(spec, buf)
    }
}

impl<T: Sample, B> Signal for SignalConsumer<T, B> {
    type Sample = T;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: Sample, B> SignalReader for SignalConsumer<T, B> {
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let (trailing, leading) = self.consumer.elements();
        if trailing.is_empty() {
            if self.consumer.is_abandoned() {
                std::sync::atomic::fence(Ordering::Acquire);
                return Ok(0);
            }

            return Err(PhonicError::not_ready());
        }

        let n_channels = self.spec.n_channels;
        let buf_len = buf.len() - buf.len() % n_channels;

        let trailing_len = trailing.len().min(buf_len);
        debug_assert_eq!(trailing_len % n_channels, 0);
        copy_to_uninit_slice(&trailing[..trailing_len], &mut buf[..trailing_len]);

        let leading_len = leading.len().min(buf_len - trailing_len);
        debug_assert_eq!(leading_len % n_channels, 0);
        let n_samples = trailing_len + leading_len;

        if leading_len > 0 {
            copy_to_uninit_slice(&leading[..leading_len], &mut buf[trailing_len..n_samples]);
        }

        self.consumer.consume(n_samples);
        Ok(n_samples)
    }
}

impl<T: Sample, B> BufferedSignalReader for SignalConsumer<T, B> {
    fn fill(&mut self) -> PhonicResult<&[Self::Sample]> {
        let (trailing, _) = self.consumer.elements();
        if trailing.is_empty() && !self.consumer.is_abandoned() {
            std::sync::atomic::fence(Ordering::Acquire);
            return Err(PhonicError::not_ready());
        }

        Ok(trailing)
    }

    fn buffer(&self) -> Option<&[Self::Sample]> {
        let (trailing, _) = self.consumer.elements();
        if trailing.is_empty() && self.consumer.is_abandoned() {
            std::sync::atomic::fence(Ordering::Acquire);
            return None;
        }

        Some(trailing)
    }

    fn consume(&mut self, n_samples: usize) {
        self.consumer.consume(n_samples)
    }
}

impl<T: Sample, B> Signal for SignalProducer<T, B> {
    type Sample = T;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: Sample, B> SignalWriter for SignalProducer<T, B> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        if self.producer.is_abandoned() {
            std::sync::atomic::fence(Ordering::Acquire);
            return Err(PhonicError::terminated());
        }

        let (trailing, leading) = self.producer.slots();
        if trailing.is_empty() {
            return Err(PhonicError::not_ready());
        }

        let n_channels = self.spec.n_channels;
        let buf_len = buf.len() - buf.len() % n_channels;

        let trailing_len = trailing.len().min(buf_len);
        debug_assert_eq!(trailing_len % n_channels, 0);
        copy_to_uninit_slice(&buf[..trailing_len], &mut trailing[..trailing_len]);

        let leading_len = leading.len().min(buf_len - trailing_len);
        debug_assert_eq!(leading_len % n_channels, 0);
        let n_samples = trailing_len + leading_len;

        if leading_len > 0 {
            copy_to_uninit_slice(&buf[trailing_len..n_samples], &mut leading[..leading_len]);
        }

        self.producer.commit(n_samples);
        Ok(n_samples)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        if self.producer.is_empty() {
            Ok(())
        } else if self.producer.is_abandoned() {
            std::sync::atomic::fence(Ordering::Acquire);
            Err(PhonicError::terminated())
        } else {
            Err(PhonicError::not_ready())
        }
    }
}

impl<T: Sample, B> BufferedSignalWriter for SignalProducer<T, B> {
    fn buffer_mut(&mut self) -> Option<&mut [MaybeUninit<Self::Sample>]> {
        if self.producer.is_abandoned() {
            std::sync::atomic::fence(Ordering::Acquire);
            return None;
        }

        let (trailing, _) = self.producer.slots();
        Some(trailing)
    }

    fn commit(&mut self, n_samples: usize) {
        self.producer.commit(n_samples)
    }
}

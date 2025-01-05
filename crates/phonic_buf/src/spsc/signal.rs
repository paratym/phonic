use crate::{
    spsc::{Consumer, Producer, SpscBuf},
    DefaultDynamicBuf, DefaultSizedBuf, DynamicBuf, OwnedBuf, SizedBuf,
};
use phonic_signal::{
    utils::copy_to_uninit_slice, BufferedSignal, BufferedSignalReader, BufferedSignalWriter,
    NSamples, PhonicError, PhonicResult, Sample, Signal, SignalDuration, SignalReader, SignalSpec,
    SignalWriter,
};
use std::mem::MaybeUninit;

pub struct SignalBuf;

pub struct SignalProducer<T, B> {
    spec: SignalSpec,
    producer: Producer<T, B>,
}

pub struct SignalConsumer<T, B> {
    spec: SignalSpec,
    consumer: Consumer<T, B>,
}

pub type SignalBufPair<T, B> = (SignalProducer<T, B>, SignalConsumer<T, B>);

impl SignalBuf {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<T, B>(spec: SignalSpec, buf: B) -> SignalBufPair<T, B>
    where
        B: AsMut<[MaybeUninit<T>]>,
    {
        let (producer, consumer) = SpscBuf::new(buf);

        (
            SignalProducer { spec, producer },
            SignalConsumer { spec, consumer },
        )
    }

    pub fn new_sized<B>(spec: SignalSpec) -> SignalBufPair<B::Item, B::Uninit>
    where
        B: SizedBuf,
        B::Uninit: AsMut<[<B::Uninit as OwnedBuf>::Item]>,
    {
        let buf = B::new_uninit();
        Self::new(spec, buf)
    }

    pub fn default_sized<T>(
        spec: SignalSpec,
    ) -> SignalBufPair<T, <DefaultSizedBuf<T> as OwnedBuf>::Uninit> {
        let buf = DefaultSizedBuf::<T>::new_uninit();
        Self::new(spec, buf)
    }

    pub fn new_duration<B>(
        spec: SignalSpec,
        duration: impl SignalDuration,
    ) -> SignalBufPair<B::Item, B::Uninit>
    where
        B: DynamicBuf,
        B::Uninit: AsMut<[MaybeUninit<B::Item>]>,
    {
        let NSamples { n_samples } = duration.into_duration(&spec);
        let buf = B::new_uninit(n_samples as usize);

        Self::new(spec, buf)
    }

    pub fn default_duration<T>(
        spec: SignalSpec,
        duration: impl SignalDuration,
    ) -> SignalBufPair<T, <DefaultDynamicBuf<T> as OwnedBuf>::Uninit> {
        let NSamples { n_samples } = duration.into_duration(&spec);
        let buf = DefaultDynamicBuf::<T>::new_uninit(n_samples as usize);

        Self::new(spec, buf)
    }
}

impl<T: Sample, B> Signal for SignalConsumer<T, B> {
    type Sample = T;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: Sample, B> Signal for SignalProducer<T, B> {
    type Sample = T;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T: Sample, B> BufferedSignal for SignalConsumer<T, B> {
    fn commit_samples(&mut self, n_samples: usize) {
        self.consumer.commit(n_samples)
    }
}

impl<T: Sample, B> BufferedSignal for SignalProducer<T, B> {
    fn commit_samples(&mut self, n_samples: usize) {
        self.producer.commit(n_samples)
    }
}

impl<T: Sample, B> SignalReader for SignalConsumer<T, B> {
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let (trailing, leading) = self.consumer.elements();
        if trailing.is_empty() {
            if self.consumer.is_abandoned() {
                return Ok(0);
            }

            return Err(PhonicError::NotReady);
        }

        let trailing_len = trailing.len().min(buf.len());
        let leading_len = leading.len().min(buf.len() - trailing_len);
        let n_samples = trailing_len + leading_len;

        copy_to_uninit_slice(&trailing[..trailing_len], &mut buf[..trailing_len]);
        copy_to_uninit_slice(&leading[..leading_len], &mut buf[trailing_len..n_samples]);
        self.consumer.commit(n_samples);

        Ok(n_samples)
    }
}

impl<T: Sample, B> BufferedSignalReader for SignalConsumer<T, B> {
    fn read_available(&self) -> &[Self::Sample] {
        let (trailing, _) = self.consumer.elements();

        trailing
    }
}

impl<T: Sample, B> SignalWriter for SignalProducer<T, B> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        if self.producer.is_abandoned() {
            return Err(PhonicError::Terminated);
        }

        let (trailing, leading) = self.producer.slots();
        let trailing_len = trailing.len().min(buf.len());
        let leading_len = trailing.len().min(buf.len() - trailing_len);
        let n_samples = trailing_len + leading_len;

        copy_to_uninit_slice(&buf[..trailing_len], &mut trailing[..trailing_len]);
        copy_to_uninit_slice(&buf[trailing_len..n_samples], &mut leading[..leading_len]);
        self.producer.commit(n_samples);

        Ok(n_samples)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        todo!()
    }
}

impl<T: Sample, B> BufferedSignalWriter for SignalProducer<T, B> {
    fn available_slots(&mut self) -> &mut [MaybeUninit<Self::Sample>] {
        let (trailing, _) = self.producer.slots();

        trailing
    }
}

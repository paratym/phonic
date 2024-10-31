use phonic_core::PhonicError;
use phonic_signal::{Sample, Signal, SignalReader, SignalSpec, SignalWriter};
use rtrb::{chunks::ChunkError, Consumer, Producer, RingBuffer};
use std::marker::PhantomData;

pub struct RealTimeSignal;
pub struct RealTimeSignalHalf<T, S> {
    spec: SignalSpec,
    inner: T,
    _sample: PhantomData<S>,
}

impl RealTimeSignal {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<S>(
        buf_cap: usize,
        spec: SignalSpec,
    ) -> (
        RealTimeSignalHalf<Producer<S>, S>,
        RealTimeSignalHalf<Consumer<S>, S>,
    ) {
        let (producer, consumer) = RingBuffer::new(buf_cap);

        (
            RealTimeSignalHalf::new(producer, spec),
            RealTimeSignalHalf::new(consumer, spec),
        )
    }
}

impl<T, S> RealTimeSignalHalf<T, S> {
    fn new(inner: T, spec: SignalSpec) -> Self {
        Self {
            inner,
            spec,
            _sample: PhantomData,
        }
    }
}

impl<T, S: Sample> Signal for RealTimeSignalHalf<T, S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<S: Sample> SignalReader for RealTimeSignalHalf<Consumer<S>, S> {
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let n_slots = self.inner.slots();
        if n_slots == 0 {
            return if self.inner.is_abandoned() {
                Ok(0)
            } else {
                Err(PhonicError::NotReady)
            };
        }

        let n_samples = buf.len().min(n_slots);
        let read_chunk = self
            .inner
            .read_chunk(n_samples)
            .map_err(|error| match error {
                ChunkError::TooFewSlots(_) => PhonicError::Unreachable,
            })?;

        buf.iter_mut().zip(read_chunk).for_each(|(a, b)| *a = b);
        Ok(n_samples)
    }
}

impl<S: Sample> SignalWriter for RealTimeSignalHalf<Producer<S>, S> {
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, PhonicError> {
        let n_slots = self.inner.slots();
        if n_slots == 0 {
            return if self.inner.is_abandoned() {
                Ok(0)
            } else {
                Err(PhonicError::NotReady)
            };
        }

        let n_samples = buf.len().min(n_slots);
        let write_chunk =
            self.inner
                .write_chunk_uninit(n_samples)
                .map_err(|error| match error {
                    ChunkError::TooFewSlots(_) => PhonicError::Unreachable,
                })?;

        write_chunk.fill_from_iter(buf[..n_samples].iter().copied());
        Ok(n_samples)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        todo!()
    }
}

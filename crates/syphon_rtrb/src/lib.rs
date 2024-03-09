use rtrb::{chunks::ChunkError, Consumer, Producer};
use std::marker::PhantomData;
use syphon_core::SyphonError;
use syphon_signal::{Sample, Signal, SignalReader, SignalSpec, SignalWriter};

pub struct RealTimeSignal<T, S> {
    spec: SignalSpec,
    inner: T,
    _sample: PhantomData<S>,
}

pub trait RingBufferHalfExt<S> {
    fn as_signal(&mut self, spec: SignalSpec) -> RealTimeSignal<&mut Self, S> {
        RealTimeSignal::new(self, spec)
    }

    fn into_signal(self, spec: SignalSpec) -> RealTimeSignal<Self, S>
    where
        Self: Sized,
    {
        RealTimeSignal::new(self, spec)
    }
}

impl<T, S> RealTimeSignal<T, S> {
    fn new(inner: T, spec: SignalSpec) -> Self {
        Self {
            inner,
            spec,
            _sample: PhantomData,
        }
    }
}

impl<S> RingBufferHalfExt<S> for Producer<S> {}
impl<S> RingBufferHalfExt<S> for Consumer<S> {}

impl<T, S: Sample> Signal for RealTimeSignal<T, S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<S: Sample> SignalReader for RealTimeSignal<Consumer<S>, S> {
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, SyphonError> {
        let n_samples = buf.len().min(self.inner.slots());
        let read_chunk = self.inner.read_chunk(n_samples).map_err(|e| match e {
            ChunkError::TooFewSlots(_) => SyphonError::Unreachable,
        })?;

        buf[..n_samples]
            .iter_mut()
            .zip(read_chunk.into_iter())
            .for_each(|(a, b)| *a = b);

        buf[n_samples..].fill(S::ORIGIN);
        Ok(n_samples)
    }
}

impl<S: Sample + Default> SignalWriter for RealTimeSignal<Producer<S>, S> {
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, SyphonError> {
        let n_slots = self.inner.slots();
        if n_slots == 0 {
            if !self.inner.is_abandoned() {
                return Err(SyphonError::NotReady);
            }

            return Ok(0);
        }

        let n_samples = buf.len().min(n_slots);
        let write_chunk = self
            .inner
            .write_chunk_uninit(n_samples)
            .map_err(|e| match e {
                ChunkError::TooFewSlots(_) => SyphonError::Unreachable,
            })?;

        write_chunk.fill_from_iter(buf[..n_samples].into_iter().copied());
        Ok(n_samples)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        todo!()
    }

    fn copy_n<R>(&mut self, reader: &mut R, mut n: u64) -> Result<(), SyphonError>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        while n > 0 {
            let n_samples = n.min(self.inner.slots() as u64);
            if n_samples == 0 {
                return Err(SyphonError::NotReady);
            }

            let mut chunk = self
                .inner
                .write_chunk(n_samples as usize)
                .map_err(|e| match e {
                    ChunkError::TooFewSlots(_) => SyphonError::Unreachable,
                })?;

            let (buf0, buf1) = chunk.as_mut_slices();
            reader.read_exact(buf0)?;
            reader.read_exact(buf1)?;
            chunk.commit_all();
            n -= n_samples;
        }

        Ok(())
    }
}

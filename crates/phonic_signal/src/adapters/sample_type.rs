use crate::{
    FiniteSignal, IndexedSignal, IntoSample, Sample, Signal, SignalReader, SignalSeeker,
    SignalSpec, SignalWriter,
};
use phonic_core::PhonicError;
use std::marker::PhantomData;

pub struct SampleTypeAdapter<T: Signal, S: Sample, const N: usize = 2048> {
    signal: T,
    buffer: [T::Sample; N],
    _sample: PhantomData<S>,
}

impl<T, S, const N: usize> SampleTypeAdapter<T, S, N>
where
    T: Signal,
    S: Sample,
{
    pub fn new(signal: T) -> Self {
        Self {
            signal,
            buffer: [T::Sample::ORIGIN; N],
            _sample: PhantomData,
        }
    }
}

impl<T, S, const N: usize> Signal for SampleTypeAdapter<T, S, N>
where
    T: Signal,
    S: Sample,
{
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        self.signal.spec()
    }
}

impl<T, S, const N: usize> IndexedSignal for SampleTypeAdapter<T, S, N>
where
    T: IndexedSignal,
    S: Sample,
{
    fn pos(&self) -> u64 {
        self.signal.pos()
    }
}

impl<T, S, const N: usize> FiniteSignal for SampleTypeAdapter<T, S, N>
where
    T: FiniteSignal,
    S: Sample,
{
    fn len(&self) -> u64 {
        self.signal.len()
    }
}

impl<T, S, const N: usize> SignalReader for SampleTypeAdapter<T, S, N>
where
    T: SignalReader,
    T::Sample: IntoSample<S>,
    S: Sample,
{
    fn read(&mut self, buffer: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let buf_len = buffer.len().min(self.buffer.len());
        let n = self.signal.read(&mut self.buffer[..buf_len])?;

        self.buffer
            .iter()
            .zip(buffer[..n].iter_mut())
            .for_each(|(inner, outer)| *outer = inner.into_sample());

        Ok(n)
    }
}

impl<T, S, const N: usize> SignalWriter for SampleTypeAdapter<T, S, N>
where
    T: SignalWriter,
    S: Sample + IntoSample<T::Sample>,
{
    fn write(&mut self, buffer: &[Self::Sample]) -> Result<usize, PhonicError> {
        let buf_len = buffer.len().min(self.buffer.len());

        self.buffer
            .iter_mut()
            .zip(buffer[..buf_len].iter())
            .for_each(|(inner, outer)| *inner = outer.into_sample());

        self.signal.write(&self.buffer[..buf_len])
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.signal.flush()
    }
}

impl<T, S, const N: usize> SignalSeeker for SampleTypeAdapter<T, S, N>
where
    T: SignalSeeker,
    S: Sample,
{
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        self.signal.seek(offset)
    }
}

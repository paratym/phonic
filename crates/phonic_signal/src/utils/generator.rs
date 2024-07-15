use crate::{Sample, Signal, SignalObserver, SignalReader, SignalSeeker, SignalSpec};
use phonic_core::PhonicError;
use std::marker::PhantomData;

pub struct SignalGenerator<S: Sample, F: Fn(u64, &mut [S]) -> Result<usize, PhonicError>> {
    spec: SignalSpec,
    callback: F,
    i: u64,
    _sample: PhantomData<S>,
}

impl<S, F> SignalGenerator<S, F>
where
    S: Sample,
    F: Fn(u64, &mut [S]) -> Result<usize, PhonicError>,
{
    pub fn new(spec: SignalSpec, callback: F) -> Self {
        Self {
            spec,
            callback,
            i: 0,
            _sample: PhantomData,
        }
    }
}

impl<S, F> Signal for SignalGenerator<S, F>
where
    S: Sample,
    F: Fn(u64, &mut [S]) -> Result<usize, PhonicError>,
{
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<S, F> SignalObserver for SignalGenerator<S, F>
where
    S: Sample,
    F: Fn(u64, &mut [S]) -> Result<usize, PhonicError>,
{
    fn position(&self) -> Result<u64, PhonicError> {
        Ok(self.i)
    }
}

impl<S, F> SignalReader for SignalGenerator<S, F>
where
    S: Sample,
    F: Fn(u64, &mut [S]) -> Result<usize, PhonicError>,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let rem_samples = self
            .spec
            .n_samples()
            .map(|n| n - self.i)
            .unwrap_or(u64::MAX);

        let len = buf.len().min(rem_samples as usize);
        let n = (self.callback)(self.i, &mut buf[..len])?;
        self.i += n as u64;
        Ok(n)
    }
}

impl<S, F> SignalSeeker for SignalGenerator<S, F>
where
    S: Sample,
    F: Fn(u64, &mut [S]) -> Result<usize, PhonicError>,
{
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        let i = self.i as i64 + offset;
        if i.is_negative() {
            return Err(PhonicError::OutOfBounds);
        }

        self.i = i as u64;
        Ok(())
    }
}

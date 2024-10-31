use crate::{IndexedSignal, Sample, Signal, SignalReader, SignalSeeker, SignalSpec};
use phonic_core::PhonicError;
use std::marker::PhantomData;

pub struct SignalGenerator<S: Sample, F: Fn(u64, &mut [S]) -> Result<usize, PhonicError>> {
    spec: SignalSpec,
    callback: F,
    pos: u64,
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
            pos: 0,
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

impl<S, F> IndexedSignal for SignalGenerator<S, F>
where
    S: Sample,
    F: Fn(u64, &mut [S]) -> Result<usize, PhonicError>,
{
    fn pos(&self) -> u64 {
        self.pos
    }
}

impl<S, F> SignalReader for SignalGenerator<S, F>
where
    S: Sample,
    F: Fn(u64, &mut [S]) -> Result<usize, PhonicError>,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let mut n = (self.callback)(self.pos, buf)?;
        let n_channels = self.spec.channels.count() as usize;
        n -= n % n_channels;

        let n_frames = n / n_channels;
        self.pos += n_frames as u64;

        Ok(n)
    }
}

impl<S, F> SignalSeeker for SignalGenerator<S, F>
where
    S: Sample,
    F: Fn(u64, &mut [S]) -> Result<usize, PhonicError>,
{
    fn seek(&mut self, mut offset: i64) -> Result<(), PhonicError> {
        offset -= offset % self.spec.channels.count() as i64;
        self.pos = self
            .pos
            .checked_add_signed(offset)
            .ok_or(PhonicError::OutOfBounds)?;

        Ok(())
    }
}

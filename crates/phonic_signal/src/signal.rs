use crate::{PhonicResult, Sample, SignalSpec};
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

pub trait Signal {
    type Sample: Sample;

    fn spec(&self) -> &SignalSpec;
}

pub trait IndexedSignal: Signal {
    /// returns the current position of this signal as a number of frames from the start.
    fn pos(&self) -> u64;

    fn pos_interleaved(&self) -> u64 {
        self.pos() * self.spec().channels.count() as u64
    }

    fn pos_duration(&self) -> Duration {
        let seconds = self.pos() as f64 / self.spec().channels.count() as f64;
        Duration::from_secs_f64(seconds)
    }
}

pub trait FiniteSignal: Signal {
    /// returns the total length of this signal as a number of frames.
    fn len(&self) -> u64;

    fn len_interleaved(&self) -> u64 {
        self.len() * self.spec().channels.count() as u64
    }

    fn len_duration(&self) -> Duration {
        let seconds = self.len() as f64 / self.spec().sample_rate as f64;
        Duration::from_secs_f64(seconds)
    }

    fn is_empty(&self) -> bool
    where
        Self: Sized + IndexedSignal,
    {
        self.pos() == self.len()
    }

    fn rem(&self) -> u64
    where
        Self: Sized + IndexedSignal,
    {
        self.len() - self.pos()
    }

    fn rem_interleaved(&self) -> u64
    where
        Self: Sized + IndexedSignal,
    {
        self.len_interleaved() - self.pos_interleaved()
    }

    fn rem_duration(&self) -> Duration
    where
        Self: Sized + IndexedSignal,
    {
        self.len_duration() - self.pos_duration()
    }
}

pub trait SignalReader: Signal {
    /// reads samples from this signal into the given buffer.
    /// returns the number of interleaved samples read.
    fn read(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize>;

    fn read_frames<'a>(
        &mut self,
        buf: &'a mut [Self::Sample],
    ) -> PhonicResult<impl Iterator<Item = &'a [Self::Sample]>>
    where
        Self: Sized,
    {
        let n = self.read(buf)?;
        let n_channels = self.spec().channels.count() as usize;
        debug_assert_eq!(n % n_channels, 0);

        Ok(buf[..n].chunks_exact(n_channels))
    }
}

pub trait SignalWriter: Signal {
    /// writes samples from the given buffer to this signal.
    /// returns the number of interleaved samples written.
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize>;

    fn flush(&mut self) -> PhonicResult<()>;
}

pub trait SignalSeeker: Signal {
    /// moves the current position of the stream by the given number of frames
    fn seek(&mut self, offset: i64) -> PhonicResult<()>;

    fn set_pos(&mut self, position: u64) -> PhonicResult<()>
    where
        Self: Sized + IndexedSignal,
    {
        let frame_offset = position as i64 - self.pos() as i64;
        self.seek(frame_offset)
    }

    fn seek_start(&mut self) -> PhonicResult<()>
    where
        Self: Sized + IndexedSignal,
    {
        self.set_pos(0)
    }

    fn seek_end(&mut self) -> PhonicResult<()>
    where
        Self: Sized + IndexedSignal + FiniteSignal,
    {
        self.set_pos(self.len())
    }
}

// TODO: call overridden methods

impl<T> Signal for T
where
    T: Deref,
    T::Target: Signal,
{
    type Sample = <T::Target as Signal>::Sample;

    fn spec(&self) -> &SignalSpec {
        self.deref().spec()
    }
}

impl<T> IndexedSignal for T
where
    T: Deref,
    T::Target: IndexedSignal,
{
    fn pos(&self) -> u64 {
        self.deref().pos()
    }
}

impl<T> FiniteSignal for T
where
    T: Deref,
    T::Target: FiniteSignal,
{
    fn len(&self) -> u64 {
        self.deref().len()
    }
}

impl<S, T> SignalReader for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalReader<Sample = S>,
{
    fn read(&mut self, buffer: &mut [S]) -> PhonicResult<usize> {
        self.deref_mut().read(buffer)
    }
}

impl<S, T> SignalWriter for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalWriter<Sample = S>,
{
    fn write(&mut self, buffer: &[S]) -> PhonicResult<usize> {
        self.deref_mut().write(buffer)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.deref_mut().flush()
    }
}

impl<S, T> SignalSeeker for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalSeeker<Sample = S>,
{
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        self.deref_mut().seek(offset)
    }
}

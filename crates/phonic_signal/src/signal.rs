use crate::{DefaultBuf, Sample, SignalSpec};
use phonic_core::PhonicError;
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

const POLL_TO_BUF_RATIO: u32 = 6;

pub trait SignalReader: Signal {
    /// reads samples from this signal into the given buffer.
    /// returns the number of interleaved samples read.
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError>;

    fn read_exact(&mut self, mut buf: &mut [Self::Sample], block: bool) -> Result<(), PhonicError> {
        let buf_len = buf.len();
        let spec = self.spec();
        if buf_len % spec.channels.count() as usize != 0 {
            return Err(PhonicError::SignalMismatch);
        }

        let poll_interval = spec.sample_rate_duration() * buf_len as u32
            / spec.channels.count()
            / POLL_TO_BUF_RATIO;

        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Err(PhonicError::Interrupted) if block => continue,
                Err(PhonicError::NotReady) if block => {
                    std::thread::sleep(poll_interval);
                    continue;
                }

                Err(e) => return Err(e),
                Ok(n) => buf = &mut buf[n..],
            }
        }

        Ok(())
    }

    fn read_frames<'a>(
        &mut self,
        buf: &'a mut [Self::Sample],
    ) -> Result<impl Iterator<Item = &'a [Self::Sample]>, PhonicError>
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
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, PhonicError>;

    fn flush(&mut self) -> Result<(), PhonicError>;

    fn write_exact(&mut self, mut buf: &[Self::Sample], block: bool) -> Result<(), PhonicError> {
        let buf_len = buf.len();
        let spec = self.spec();
        if buf_len % spec.channels.count() as usize != 0 {
            return Err(PhonicError::SignalMismatch);
        }

        let poll_interval = spec.sample_rate_duration() * buf_len as u32
            / spec.channels.count()
            / POLL_TO_BUF_RATIO;

        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Err(PhonicError::Interrupted) if block => continue,
                Err(PhonicError::NotReady) if block => {
                    std::thread::sleep(poll_interval);
                    continue;
                }

                Err(e) => return Err(e),
                Ok(n) => buf = &buf[n..],
            };
        }

        Ok(())
    }

    /// copies a given number of frames from a `SignalReader` to this signal via a given buffer.
    fn copy_n_buffered<R>(
        &mut self,
        reader: &mut R,
        n_frames: u64,
        buf: &mut [Self::Sample],
        block: bool,
    ) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        let spec = self.spec();
        let n_channels = spec.channels.count();
        let buf_len = buf.len();

        if !spec.is_compatible(reader.spec()) || buf_len < n_channels as usize {
            return Err(PhonicError::SignalMismatch);
        }

        let n_samples = n_frames.saturating_mul(n_channels as u64);
        let mut n = 0;

        let poll_interval =
            self.spec().sample_rate_duration() * buf_len as u32 / n_channels / POLL_TO_BUF_RATIO;

        while n < n_samples {
            let len = buf_len.min((n_samples - n) as usize);
            let n_read = match reader.read(&mut buf[..len]) {
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Err(PhonicError::Interrupted) if block => continue,
                Err(PhonicError::NotReady) if block => {
                    std::thread::sleep(poll_interval);
                    continue;
                }

                Err(e) => return Err(e),
                Ok(n) => n,
            };

            self.write_exact(&buf[..n_read], block)?;
            n += n_read as u64;
        }

        Ok(())
    }

    /// copies a given number of frames from a `SignalReader` directly to this signal.
    /// if this method isn't implemented it falls back to copying via a stack allocated
    /// buffer.
    fn copy_n<R>(&mut self, reader: &mut R, n_frames: u64, block: bool) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        let mut buf = DefaultBuf::default();
        self.copy_n_buffered(reader, n_frames, &mut buf, block)
    }

    fn copy_all_buffered<R>(
        &mut self,
        reader: &mut R,
        buf: &mut [Self::Sample],
        block: bool,
    ) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        match self.copy_n_buffered(reader, u64::MAX, buf, block) {
            Err(PhonicError::OutOfBounds) => Ok(()),
            result => result,
        }
    }

    fn copy_all<R>(&mut self, reader: &mut R, block: bool) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        let mut buf = DefaultBuf::default();
        self.copy_all_buffered(reader, &mut buf, block)
    }
}

pub trait SignalSeeker: Signal {
    /// moves the current position of the stream by the given number of frames
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError>;

    fn set_pos(&mut self, position: u64) -> Result<(), PhonicError>
    where
        Self: Sized + IndexedSignal,
    {
        let frame_offset = position as i64 - self.pos() as i64;
        self.seek(frame_offset)
    }

    fn seek_start(&mut self) -> Result<(), PhonicError>
    where
        Self: Sized + IndexedSignal,
    {
        self.set_pos(0)
    }

    fn seek_end(&mut self) -> Result<(), PhonicError>
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
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, PhonicError> {
        self.deref_mut().read(buffer)
    }
}

impl<S, T> SignalWriter for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalWriter<Sample = S>,
{
    fn write(&mut self, buffer: &[S]) -> Result<usize, PhonicError> {
        self.deref_mut().write(buffer)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.deref_mut().flush()
    }
}

impl<S, T> SignalSeeker for T
where
    S: Sample,
    T: DerefMut,
    T::Target: SignalSeeker<Sample = S>,
{
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        self.deref_mut().seek(offset)
    }
}

use crate::{PhonicResult, Sample, SignalSpec};
use phonic_macro::impl_deref_signal;
use std::{
    ops::{Deref, DerefMut, Neg},
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

    fn set_pos(&mut self, pos: u64) -> PhonicResult<()>
    where
        Self: Sized + IndexedSignal,
    {
        let current_pos = self.pos();
        let offset = if pos >= current_pos {
            (pos - current_pos) as i64
        } else {
            ((current_pos - pos) as i64).neg()
        };

        self.seek(offset)
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

impl_deref_signal! {
    impl<T: Deref> _ for T {
        type Target = T::Target;

        &self -> self.deref();

        &mut self -> self.deref_mut()
        where
            T: DerefMut;
    }
}

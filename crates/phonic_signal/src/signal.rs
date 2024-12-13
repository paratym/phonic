use phonic_macro::delegate_group;
use std::ops::{Deref, DerefMut};

delegate_group! {
    #![mod_path(crate)]

    pub trait Signal {
        type Sample: crate::Sample;

        fn spec(&self) -> &crate::SignalSpec;
    }

    pub trait IndexedSignal: Signal {
        /// returns the current position of this signal as a number of frames from the start.
        fn pos(&self) -> u64;

        fn pos_interleaved(&self) -> u64 {
            self.pos() * self.spec().channels.count() as u64
        }

        fn pos_duration(&self) -> std::time::Duration {
            let seconds = self.pos() as f64 / self.spec().channels.count() as f64;
            std::time::Duration::from_secs_f64(seconds)
        }
    }

    pub trait FiniteSignal: Signal {
        /// returns the total length of this signal as a number of frames.
        fn len(&self) -> u64;

        fn len_interleaved(&self) -> u64 {
            self.len() * self.spec().channels.count() as u64
        }

        fn len_duration(&self) -> std::time::Duration {
            let seconds = self.len() as f64 / self.spec().sample_rate as f64;
            std::time::Duration::from_secs_f64(seconds)
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

        fn rem_duration(&self) -> std::time::Duration
        where
            Self: Sized + IndexedSignal,
        {
            self.len_duration() - self.pos_duration()
        }
    }

    #[subgroup(Mut, Read)]
    pub trait SignalReader: Signal {
        /// reads samples from this signal into the given buffer.
        /// returns the number of interleaved samples read.
        fn read(&mut self, buf: &mut [std::mem::MaybeUninit<Self::Sample>]) -> crate::PhonicResult<usize>;

        fn read_init<'a>(
            &mut self,
            buf: &'a mut [std::mem::MaybeUninit<Self::Sample>],
        ) -> crate::PhonicResult<&'a mut [Self::Sample]> {
            let n_samples = self.read(buf)?;
            let uninit_slice = &mut buf[..n_samples];
            let init_slice = unsafe { crate::utils::slice_as_init_mut(uninit_slice) };

            Ok(init_slice)
        }

        fn read_frames<'a>(
            &mut self,
            buf: &'a mut [std::mem::MaybeUninit<Self::Sample>],
        ) -> crate::PhonicResult<impl Iterator<Item = &'a [Self::Sample]>>
        where
            Self: Sized,
        {
            let samples = self.read_init(buf)?;
            let n_channels = self.spec().channels.count() as usize;
            debug_assert_eq!(samples.len() % n_channels, 0);

            Ok(samples.chunks_exact(n_channels))
        }
    }

    #[subgroup(Mut, Write)]
    pub trait SignalWriter: Signal {
        /// writes samples from the given buffer to this signal.
        /// returns the number of interleaved samples written.
        fn write(&mut self, buf: &[Self::Sample]) -> crate::PhonicResult<usize>;

        fn flush(&mut self) -> crate::PhonicResult<()>;
    }

    #[subgroup(Mut)]
    pub trait SignalSeeker: Signal {
        /// moves the current position of the stream by the given number of frames
        fn seek(&mut self, offset: i64) -> crate::PhonicResult<()>;

        fn set_pos(&mut self, pos: u64) -> crate::PhonicResult<()>
        where
            Self: Sized + IndexedSignal,
        {
            let current_pos = self.pos();
            let offset = if pos >= current_pos {
                (pos - current_pos) as i64
            } else {
                -((current_pos - pos) as i64)
            };

            self.seek(offset)
        }

        fn seek_start(&mut self) -> crate::PhonicResult<()>
        where
            Self: Sized + IndexedSignal,
        {
            self.set_pos(0)
        }

        fn seek_end(&mut self) -> crate::PhonicResult<()>
        where
            Self: Sized + IndexedSignal + FiniteSignal,
        {
            self.set_pos(self.len())
        }
    }
}

delegate_signal! {
    delegate<T> * for T {
        Self as T::Target;

        &self => self.deref()
        where T: Deref;

        &mut self => self.deref_mut()
        where T: DerefMut;
    }
}

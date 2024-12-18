use phonic_macro::delegate_group;
use std::ops::{Deref, DerefMut};

delegate_group! {
    #![mod_path(crate)]

    pub trait Signal {
        type Sample: crate::Sample;

        fn spec(&self) -> &crate::SignalSpec;
    }

    #[subgroup(Mut, Buffered)]
    pub trait BufferedSignal: Signal {
        fn commit_samples(&mut self, n_samples: usize);
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

    #[subgroup(Mut, Read, Buffered)]
    pub trait BufferedSignalReader: BufferedSignal + SignalReader {
        fn available_samples(&self) -> &[Self::Sample];

        fn read_available(&mut self, buf: &mut [std::mem::MaybeUninit<Self::Sample>]) -> usize {
            let mut n_samples = 0;
            let buf_len = buf.len();

            while n_samples < buf_len {
                let available = self.available_samples();
                let available_len = available.len();
                if available_len == 0 {
                    break;
                }

                let slice_len = available_len.min(buf_len - n_samples);
                let src = &available[..slice_len];
                let dst = &mut buf[n_samples..n_samples + slice_len];

                crate::utils::copy_to_uninit_slice(src, dst);
                self.commit_samples(slice_len);
                n_samples += slice_len;
            }

            n_samples
        }

        fn read_init_available<'a>(
            &mut self,
            buf: &'a mut [std::mem::MaybeUninit<Self::Sample>],
        ) -> &'a mut [Self::Sample] {
            let n_samples = self.read_available(buf);
            let uninit_slice = &mut buf[..n_samples];
            unsafe { crate::utils::slice_as_init_mut(uninit_slice) }
        }
    }

    #[subgroup(Mut, Read, Blocking)]
    pub trait BlockingSignalReader: SignalReader {
        fn read_blocking(&mut self, buf: &mut [std::mem::MaybeUninit<Self::Sample>]) -> crate::PhonicResult<usize>;

        fn read_init_blocking<'a>(
            &mut self,
            buf: &'a mut [std::mem::MaybeUninit<Self::Sample>],
        ) -> crate::PhonicResult<&'a mut [Self::Sample]> {
            let n_samples = self.read_blocking(buf)?;
            let uninit_slice = &mut buf[..n_samples];
            let init_slice = unsafe { crate::utils::slice_as_init_mut(uninit_slice) };

            Ok(init_slice)
        }

        fn read_exact(
            &mut self,
            buf: &mut [std::mem::MaybeUninit<Self::Sample>],
        ) -> crate::PhonicResult<()> {
            let buf_len = buf.len();
            if buf_len % self.spec().channels.count() as usize != 0 {
                return Err(crate::PhonicError::InvalidInput);
            }

            let mut i = 0;
            while i < buf_len {
                match self.read_blocking(&mut buf[i..]) {
                    Err(crate::PhonicError::Interrupted | crate::PhonicError::NotReady) => continue,
                    Err(e) => return Err(e),
                    Ok(0) => return Err(crate::PhonicError::OutOfBounds),
                    Ok(n) => i += n,
                }
            }

            Ok(())
        }

        fn read_exact_init<'a>(
            &mut self,
            buf: &'a mut [std::mem::MaybeUninit<Self::Sample>],
        ) -> crate::PhonicResult<&'a mut [Self::Sample]> {
            self.read_exact(buf)?;
            Ok(unsafe { crate::utils::slice_as_init_mut(buf) })
        }
    }

    #[subgroup(Mut, Write)]
    pub trait SignalWriter: Signal {
        /// writes samples from the given buffer to this signal.
        /// returns the number of interleaved samples written.
        fn write(&mut self, buf: &[Self::Sample]) -> crate::PhonicResult<usize>;

        fn flush(&mut self) -> crate::PhonicResult<()>;
    }

    #[subgroup(Mut, Write, Buffered)]
    pub trait BufferedSignalWriter: BufferedSignal + SignalWriter {
        fn available_slots(&mut self) -> &mut [std::mem::MaybeUninit<Self::Sample>];

        fn write_available(&mut self, buf: &[Self::Sample]) -> usize {
            let mut n_samples = 0;
            let buf_len = buf.len();

            while n_samples < buf_len {
                let available = self.available_slots();
                let available_len = available.len();
                if available_len == 0 {
                    break;
                }

                let slice_len = available_len.min(buf_len - n_samples);
                let buf_slice = &buf[n_samples..n_samples + slice_len];
                let slot_ptr = available.as_mut_ptr().cast();

                unsafe {
                    buf_slice
                        .as_ptr()
                        .copy_to_nonoverlapping(slot_ptr, slice_len);
                }

                self.commit_samples(slice_len);
                n_samples += slice_len;
            }

            n_samples
        }
    }

    #[subgroup(Mut, Write, Blocking)]
    pub trait BlockingSignalWriter: SignalWriter {
        fn write_blocking(&mut self, buf: &[Self::Sample]) -> crate::PhonicResult<usize>;
        fn flush_blocking(&mut self) -> crate::PhonicResult<()>;

        fn write_exact(&mut self, mut buf: &[Self::Sample]) -> crate::PhonicResult<()> {
            if buf.len() % self.spec().channels.count() as usize != 0 {
                return Err(crate::PhonicError::InvalidInput);
            }

            while !buf.is_empty() {
                match self.write_blocking(buf) {
                    Err(crate::PhonicError::Interrupted | crate::PhonicError::NotReady) => continue,
                    Err(e) => return Err(e),
                    Ok(0) => return Err(crate::PhonicError::OutOfBounds),
                    Ok(n) => buf = &buf[n..],
                };
            }

            Ok(())
        }
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

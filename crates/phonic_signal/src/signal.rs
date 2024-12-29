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
        fn pos(&self) -> u64;

        fn pos_duration<D: crate::SignalDuration>(&self) -> D
        where
            Self: Sized
        {
            use crate::IntoDuration;
            crate::NFrames::from(self.pos()).into_duration(self.spec())
        }
    }

    pub trait FiniteSignal: Signal {
        fn len(&self) -> u64;

        fn len_duration<D: crate::SignalDuration>(&self) -> D
        where
            Self: Sized
        {
            use crate::IntoDuration;
            crate::NFrames::from(self.len()).into_duration(self.spec())
        }

        fn is_empty(&self) -> bool
        where
            Self: Sized + IndexedSignal,
        {
            self.pos() == self.len()
        }

        fn rem(&self) -> u64
        where
            Self: Sized + IndexedSignal
        {
            self.len() - self.pos()
        }

        fn rem_duration<D>(&self) -> D
        where
            Self: Sized + IndexedSignal,
            D: crate::SignalDuration
        {
            use crate::IntoDuration;
            crate::NFrames::from(self.rem()).into_duration(self.spec())
        }
    }

    #[subgroup(Mut, Read)]
    pub trait SignalReader: Signal {
        /// reads samples from this signal into the given buffer.
        /// returns the number of interleaved samples read.
        fn read(
            &mut self,
            buf: &mut [std::mem::MaybeUninit<Self::Sample>]
        ) -> crate::PhonicResult<usize>;

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
        fn read_blocking(
            &mut self,
            buf: &mut [std::mem::MaybeUninit<Self::Sample>]
        ) -> crate::PhonicResult<usize>;

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

            let init_slice = unsafe { crate::utils::slice_as_init_mut(buf) };
            Ok(init_slice)
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
                let src = &buf[n_samples..n_samples + slice_len];
                let dst = &mut available[..slice_len];

                crate::utils::copy_to_uninit_slice(src, dst);

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
        fn seek(&mut self, offset: i64) -> crate::PhonicResult<()>;

        fn seek_forward<D>(&mut self, offset: D) -> crate::PhonicResult<()>
        where
            Self: Sized,
            D: crate::SignalDuration
        {
            let crate::NFrames { n_frames } = offset.into_duration(self.spec());
            self.seek(n_frames as i64)
        }

        fn seek_backward<D>(&mut self, offset: D) -> crate::PhonicResult<()>
        where
            Self: Sized,
            D: crate::SignalDuration
        {
            let crate::NFrames { n_frames } = offset.into_duration(self.spec());
            self.seek(-(n_frames as i64))
        }

        fn seek_from_start<D>(&mut self, duration: D) -> crate::PhonicResult<()>
        where
            Self: Sized + IndexedSignal,
            D: crate::SignalDuration
        {
            let crate::NFrames { n_frames: pos } = self.pos_duration();
            let crate::NFrames { n_frames: new_pos } = duration.into_duration(self.spec());

            let offset = if new_pos >= pos {
                (new_pos - pos) as i64
            } else {
                -((pos - new_pos) as i64)
            };

            self.seek(offset)
        }

        fn seek_to_start(&mut self) -> crate::PhonicResult<()>
        where
            Self: Sized + IndexedSignal,
        {
            self.seek_from_start(crate::NFrames::from(0))
        }

        fn seek_from_end<D>(&mut self, duration: D) -> crate::PhonicResult<()>
        where
            Self: Sized + IndexedSignal + FiniteSignal,
            D: crate::SignalDuration
        {
            let crate::NFrames { n_frames } = duration.into_duration(self.spec());
            let new_pos: crate::NFrames = self.len()
                .checked_sub(n_frames)
                .ok_or(crate::PhonicError::OutOfBounds)?
                .into();

            self.seek_from_start(new_pos)
        }

        fn seek_to_end(&mut self) -> crate::PhonicResult<()>
        where
            Self: Sized + IndexedSignal + FiniteSignal
        {
            self.seek_from_end(crate::NFrames::from(0))
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

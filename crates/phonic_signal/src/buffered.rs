use crate::{utils::slice_as_init_mut, Signal, SignalReader, SignalWriter};
use std::{mem::MaybeUninit, ops::DerefMut};

pub trait BufferedSignal: Signal {
    fn commit_samples(&mut self, n_samples: usize);
}

pub trait BufferedSignalReader: BufferedSignal + SignalReader {
    fn available_samples(&self) -> &[MaybeUninit<Self::Sample>];

    fn read_available(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> usize {
        let mut n_samples = 0;
        let buf_len = buf.len();

        while n_samples < buf_len {
            let available = self.available_samples();
            let available_len = available.len();
            if available_len == 0 {
                break;
            }

            let slice_len = available_len.min(buf_len - n_samples);
            buf[n_samples..n_samples + slice_len].copy_from_slice(&available[..slice_len]);
            self.commit_samples(slice_len);
            n_samples += slice_len;
        }

        n_samples
    }

    fn read_init_available<'a>(
        &mut self,
        buf: &'a mut [MaybeUninit<Self::Sample>],
    ) -> &'a mut [Self::Sample] {
        let n_samples = self.read_available(buf);
        let uninit_slice = &mut buf[..n_samples];
        unsafe { slice_as_init_mut(uninit_slice) }
    }
}

pub trait BufferedSignalWriter: BufferedSignal + SignalWriter {
    fn available_slots(&mut self) -> &mut [MaybeUninit<Self::Sample>];

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

pub trait BufferedSignalCopy<R>
where
    Self: Sized + BufferedSignalWriter,
    R: BufferedSignalReader<Sample = Self::Sample>,
{
    fn copy_available(&mut self, reader: &mut R) -> usize {
        let mut n_samples = 0;

        loop {
            let samples = reader.available_samples();
            let samples_len = samples.len();
            if samples_len == 0 {
                break;
            }

            let slots = self.available_slots();
            let slots_len = slots.len();
            if slots_len == 0 {
                break;
            }

            let slice_len = samples_len.min(slots_len);
            let slot_ptr = slots.as_mut_ptr().cast();

            unsafe { samples.as_ptr().copy_to_nonoverlapping(slot_ptr, slice_len) }

            reader.commit_samples(slice_len);
            self.commit_samples(slice_len);
            n_samples += slice_len;
        }

        n_samples
    }
}

impl<T> BufferedSignal for T
where
    T: DerefMut,
    T::Target: BufferedSignal,
{
    fn commit_samples(&mut self, n_samples: usize) {
        self.deref_mut().commit_samples(n_samples);
    }
}

impl<T> BufferedSignalReader for T
where
    T: DerefMut,
    T::Target: BufferedSignalReader,
{
    fn available_samples(&self) -> &[MaybeUninit<Self::Sample>] {
        self.deref().available_samples()
    }
}

impl<T> BufferedSignalWriter for T
where
    T: DerefMut,
    T::Target: BufferedSignalWriter,
{
    fn available_slots(&mut self) -> &mut [MaybeUninit<Self::Sample>] {
        self.deref_mut().available_slots()
    }
}

impl<T, R> BufferedSignalCopy<R> for T
where
    T: Sized + BufferedSignalWriter,
    R: BufferedSignalReader<Sample = T::Sample>,
{
}

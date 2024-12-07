use crate::{
    utils::{slice_as_init_mut, DefaultBuf},
    PhonicError, PhonicResult, Signal,
};
use std::mem::MaybeUninit;

pub trait BlockingSignalReader: Signal {
    fn read_blocking(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize>;

    fn read_init_blocking<'a>(
        &mut self,
        buf: &'a mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<&'a mut [Self::Sample]> {
        let n_samples = self.read_blocking(buf)?;
        let uninit_slice = &mut buf[..n_samples];
        let init_slice = unsafe { slice_as_init_mut(uninit_slice) };

        Ok(init_slice)
    }

    fn read_exact_blocking<'a>(
        &mut self,
        buf: &'a mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<&'a mut [Self::Sample]> {
        let buf_len = buf.len();
        if buf_len % self.spec().channels.count() as usize != 0 {
            return Err(PhonicError::InvalidInput);
        }

        let mut i = 0;
        while i < buf_len {
            match self.read_blocking(&mut buf[i..]) {
                Err(PhonicError::Interrupted | PhonicError::NotReady) => continue,
                Err(e) => return Err(e),
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => i += n,
            }
        }

        let init_buf = unsafe { slice_as_init_mut(buf) };
        Ok(init_buf)
    }
}

pub trait BlockingSignalWriter: Signal {
    fn write_blocking(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize>;
    fn flush_blocking(&mut self) -> PhonicResult<()>;

    fn write_exact_blocking(&mut self, mut buf: &[Self::Sample]) -> PhonicResult<()> {
        if buf.len() % self.spec().channels.count() as usize != 0 {
            return Err(PhonicError::InvalidInput);
        }

        while !buf.is_empty() {
            match self.write_blocking(buf) {
                Err(PhonicError::Interrupted | PhonicError::NotReady) => continue,
                Err(e) => return Err(e),
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => buf = &buf[n..],
            };
        }

        Ok(())
    }
}

pub trait BlockingSignalCopy<R>
where
    Self: Sized + BlockingSignalWriter,
    R: BlockingSignalReader<Sample = Self::Sample>,
{
    fn copy_n_buffered_blocking(
        &mut self,
        reader: &mut R,
        n_frames: u64,
        buf: &mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<()> {
        let spec = self.spec().merged(reader.spec())?;
        let n_samples = n_frames * spec.channels.count() as u64;
        let mut n = 0;

        while n < n_samples {
            let len = buf.len().min((n_samples - n) as usize);
            let samples = match reader.read_init_blocking(&mut buf[..len]) {
                Err(PhonicError::Interrupted | PhonicError::NotReady) => continue,
                Err(e) => return Err(e),
                Ok([]) => return Err(PhonicError::OutOfBounds),
                Ok(samples) => samples,
            };

            self.write_exact_blocking(samples)?;
            n += samples.len() as u64;
        }

        Ok(())
    }

    fn copy_n_blocking(&mut self, reader: &mut R, n_frames: u64) -> PhonicResult<()> {
        let mut buf = <DefaultBuf<_>>::default();
        self.copy_n_buffered_blocking(reader, n_frames, &mut buf)
    }

    fn copy_all_buffered_blocking(
        &mut self,
        reader: &mut R,
        buf: &mut [MaybeUninit<Self::Sample>],
    ) -> PhonicResult<()> {
        let _ = self.spec().merged(reader.spec())?;

        loop {
            let samples = match reader.read_init_blocking(buf) {
                Err(PhonicError::Interrupted | PhonicError::NotReady) => continue,
                Err(e) => return Err(e),
                Ok([]) => break,
                Ok(samples) => samples,
            };

            match self.write_exact_blocking(samples) {
                Ok(()) => continue,
                Err(PhonicError::OutOfBounds) => break,
                Err(e) => return Err(e),
            };
        }

        Ok(())
    }

    fn copy_all_blocking(&mut self, reader: &mut R) -> PhonicResult<()> {
        let mut buf = <DefaultBuf<_>>::default();
        self.copy_all_buffered_blocking(reader, &mut buf)
    }
}

impl<T, R> BlockingSignalCopy<R> for T
where
    T: Sized + BlockingSignalWriter,
    R: BlockingSignalReader<Sample = Self::Sample>,
{
}

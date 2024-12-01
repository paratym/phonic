use crate::{utils::DefaultBuf, PhonicError, PhonicResult, Signal};

pub trait BlockingSignalReader: Signal {
    fn read_blocking(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize>;

    fn read_exact_blocking(&mut self, mut buf: &mut [Self::Sample]) -> PhonicResult<()> {
        if buf.len() % self.spec().channels.count() as usize != 0 {
            return Err(PhonicError::InvalidInput);
        }

        while !buf.is_empty() {
            match self.read_blocking(buf) {
                Err(PhonicError::Interrupted | PhonicError::NotReady) => continue,
                Err(e) => return Err(e),
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => buf = &mut buf[n..],
            }
        }

        Ok(())
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
        buf: &mut [Self::Sample],
    ) -> PhonicResult<()> {
        let spec = self.spec().merged(reader.spec())?;
        let n_samples = n_frames * spec.channels.count() as u64;
        let mut n = 0;

        while n < n_samples {
            let len = buf.len().min((n_samples - n) as usize);
            let n_read = match reader.read_blocking(&mut buf[..len]) {
                Err(PhonicError::Interrupted | PhonicError::NotReady) => continue,
                Err(e) => return Err(e),
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => n,
            };

            self.write_exact_blocking(&buf[..n_read])?;
            n += n_read as u64;
        }

        Ok(())
    }

    fn copy_n_blocking(&mut self, reader: &mut R, n_frames: u64) -> PhonicResult<()> {
        let mut buf = DefaultBuf::default();
        self.copy_n_buffered_blocking(reader, n_frames, &mut buf)
    }

    fn copy_all_buffered_blocking(
        &mut self,
        reader: &mut R,
        buf: &mut [Self::Sample],
    ) -> PhonicResult<()> {
        let _ = self.spec().merged(reader.spec())?;

        loop {
            let n_read = match reader.read_blocking(buf) {
                Err(PhonicError::Interrupted | PhonicError::NotReady) => continue,
                Err(e) => return Err(e),
                Ok(0) => break,
                Ok(n) => n,
            };

            match self.write_exact_blocking(&buf[..n_read]) {
                Ok(()) => continue,
                Err(PhonicError::OutOfBounds) => break,
                Err(e) => return Err(e),
            };
        }

        Ok(())
    }

    fn copy_all_blocking(&mut self, reader: &mut R) -> PhonicResult<()> {
        let mut buf = DefaultBuf::default();
        self.copy_all_buffered_blocking(reader, &mut buf)
    }
}

impl<T, R> BlockingSignalCopy<R> for T
where
    T: Sized + BlockingSignalWriter,
    R: BlockingSignalReader<Sample = Self::Sample>,
{
}

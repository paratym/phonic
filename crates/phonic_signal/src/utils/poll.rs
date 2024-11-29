use crate::{utils::DefaultBuf, PhonicError, PhonicResult, Signal, SignalReader, SignalWriter};

pub trait PollSignal: Signal {
    fn poll_interval();
}

pub trait PollSignalReader: PollSignal + SignalReader {
    fn read_poll(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize> {
        loop {
            match self.read(buf) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => Self::poll_interval(),
                result => return result,
            }
        }
    }

    fn read_exact(&mut self, mut buf: &mut [Self::Sample]) -> PhonicResult<()> {
        let spec = self.spec();
        if buf.len() % spec.channels.count() as usize != 0 {
            return Err(PhonicError::InvalidInput);
        }

        while !buf.is_empty() {
            match self.read(buf) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => Self::poll_interval(),
                Err(e) => return Err(e),
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => buf = &mut buf[n..],
            }
        }

        Ok(())
    }
}

pub trait PollSignalWriter: PollSignal + SignalWriter {
    fn write_poll(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        loop {
            match self.write(buf) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => Self::poll_interval(),
                result => return result,
            }
        }
    }

    fn write_exact(&mut self, mut buf: &[Self::Sample]) -> PhonicResult<()> {
        let spec = self.spec();
        if buf.len() % spec.channels.count() as usize != 0 {
            return Err(PhonicError::InvalidInput);
        }

        while !buf.is_empty() {
            match self.write(buf) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => Self::poll_interval(),
                Err(e) => return Err(e),
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => buf = &buf[n..],
            };
        }

        Ok(())
    }

    fn copy_n_buffered<R>(
        &mut self,
        reader: &mut R,
        n_frames: u64,
        buf: &mut [Self::Sample],
    ) -> PhonicResult<()>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        let spec = self.spec().merged(reader.spec())?;
        let n_samples = n_frames * spec.channels.count() as u64;
        let mut n = 0;

        while n < n_samples {
            let len = buf.len().min((n_samples - n) as usize);
            let n_read = match reader.read(&mut buf[..len]) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => {
                    Self::poll_interval();
                    continue;
                }

                Err(e) => return Err(e),
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => n,
            };

            self.write_exact(&buf[..n_read])?;
            n += n_read as u64;
        }

        Ok(())
    }

    fn copy_n<R>(&mut self, reader: &mut R, n_frames: u64) -> PhonicResult<()>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        let mut buf = DefaultBuf::default();
        self.copy_n_buffered(reader, n_frames, &mut buf)
    }

    fn copy_all_buffered<R>(&mut self, reader: &mut R, buf: &mut [Self::Sample]) -> PhonicResult<()>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        let _ = self.spec().merged(reader.spec())?;

        loop {
            let n_read = match reader.read(buf) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => {
                    Self::poll_interval();
                    continue;
                }

                Err(e) => return Err(e),
                Ok(0) => break,
                Ok(n) => n,
            };

            match self.write_exact(&buf[..n_read]) {
                Ok(()) => continue,
                Err(PhonicError::OutOfBounds) => break, // TODO: write remainder
                Err(e) => return Err(e),
            };
        }

        Ok(())
    }

    fn copy_all<R>(&mut self, reader: &mut R) -> PhonicResult<()>
    where
        Self: Sized,
        R: SignalReader<Sample = Self::Sample>,
    {
        let mut buf = DefaultBuf::default();
        self.copy_all_buffered(reader, &mut buf)
    }
}

impl<T: Signal> PollSignal for T {
    fn poll_interval() {
        todo!()
    }
}

impl<T: SignalReader> PollSignalReader for T {}
impl<T: SignalWriter> PollSignalWriter for T {}

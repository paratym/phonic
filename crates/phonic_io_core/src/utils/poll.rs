use crate::{Stream, StreamReader, StreamWriter};
use phonic_signal::{utils::DefaultBuf, PhonicError, PhonicResult};

pub trait PollStream: Stream {
    fn poll_interval();
}

pub trait PollStreamReader: PollStream + StreamReader {
    fn read_poll(&mut self, buf: &mut [u8]) -> PhonicResult<usize> {
        loop {
            match self.read(buf) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => Self::poll_interval(),
                result => return result,
            }
        }
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> PhonicResult<()> {
        let buf_len = buf.len();
        if buf_len % self.stream_spec().block_align != 0 {
            return Err(PhonicError::InvalidInput);
        }

        while !buf.is_empty() {
            match self.read(buf) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => {
                    Self::poll_interval();
                    continue;
                }

                Err(e) => return Err(e),
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Ok(n) => buf = &mut buf[n..],
            };
        }

        Ok(())
    }
}

pub trait PollStreamWriter: PollStream + StreamWriter {
    fn write_poll(&mut self, buf: &[u8]) -> PhonicResult<usize> {
        loop {
            match self.write(buf) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => Self::poll_interval(),
                result => return result,
            }
        }
    }

    fn write_exact(&mut self, mut buf: &[u8]) -> PhonicResult<()> {
        let buf_len = buf.len();
        if buf_len % self.stream_spec().block_align != 0 {
            return Err(PhonicError::InvalidInput);
        }

        while !buf.is_empty() {
            match self.write(buf) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => {
                    Self::poll_interval();
                    continue;
                }

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
        n_bytes: usize,
        buf: &mut [u8],
    ) -> PhonicResult<()>
    where
        Self: Sized,
        R: StreamReader,
        R::Tag: TryInto<Self::Tag>,
        PhonicError: From<<R::Tag as TryInto<Self::Tag>>::Error>,
    {
        let spec = self.stream_spec().merged(reader.stream_spec())?;
        let buf_len = buf.len();

        if n_bytes % spec.block_align != 0 || buf_len < spec.block_align {
            return Err(PhonicError::InvalidInput);
        }

        let mut n = 0;
        while n < n_bytes {
            let len = buf_len.min(n_bytes - n);
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
            n += n_read;
        }

        Ok(())
    }

    fn copy_n<R>(&mut self, reader: &mut R, n_bytes: usize) -> PhonicResult<()>
    where
        Self: Sized,
        R: StreamReader,
        R::Tag: TryInto<Self::Tag>,
        PhonicError: From<<R::Tag as TryInto<Self::Tag>>::Error>,
    {
        let mut buf = DefaultBuf::default();
        self.copy_n_buffered(reader, n_bytes, &mut buf)
    }

    fn copy_all_buffered<R>(&mut self, reader: &mut R, buf: &mut [u8]) -> PhonicResult<()>
    where
        Self: Sized,
        R: StreamReader,
        R::Tag: TryInto<Self::Tag>,
        PhonicError: From<<R::Tag as TryInto<Self::Tag>>::Error>,
    {
        let _ = self.stream_spec().merged(reader.stream_spec())?;

        loop {
            let n_read = match reader.read(buf) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => {
                    Self::poll_interval();
                    continue;
                }

                Err(e) => return Err(e),
                Ok(0) => return Ok(()),
                Ok(n) => n,
            };

            match self.write_exact(&buf[..n_read]) {
                Ok(()) => continue,
                Err(PhonicError::OutOfBounds) => return Ok(()), // TODO: write remainder
                Err(e) => return Err(e),
            }
        }
    }

    fn copy_all<R>(&mut self, reader: &mut R) -> PhonicResult<()>
    where
        Self: Sized,
        R: StreamReader,
        R::Tag: TryInto<Self::Tag>,
        PhonicError: From<<R::Tag as TryInto<Self::Tag>>::Error>,
    {
        let mut buf = DefaultBuf::default();
        self.copy_all_buffered(reader, &mut buf)
    }
}

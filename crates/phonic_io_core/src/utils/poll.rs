use crate::{Stream, StreamReader, StreamWriter};
use phonic_signal::{utils::DefaultBuf, PhonicError, PhonicResult};
use std::mem::{transmute, MaybeUninit};

pub trait PollStream: Stream {
    fn poll_interval() {
        todo!()
    }
}

pub trait PollStreamReader: PollStream + StreamReader {
    fn read_poll(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize> {
        loop {
            match self.read(buf) {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => Self::poll_interval(),
                result => return result,
            }
        }
    }

    fn read_exact_poll(&mut self, mut buf: &mut [MaybeUninit<u8>]) -> PhonicResult<()> {
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

    fn write_exact_poll(&mut self, mut buf: &[u8]) -> PhonicResult<()> {
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
}

pub trait PollStreamCopy<R>
where
    Self: Sized + PollStreamWriter,
    R: PollStreamReader,
    R::Tag: TryInto<Self::Tag>,
    PhonicError: From<<R::Tag as TryInto<Self::Tag>>::Error>,
{
    fn copy_n_buffered_poll(
        &mut self,
        reader: &mut R,
        n_bytes: usize,
        buf: &mut [MaybeUninit<u8>],
    ) -> PhonicResult<()> {
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

            let uninit_slice = &buf[..n_read];
            let init_slice = unsafe { transmute::<&[MaybeUninit<u8>], &[u8]>(uninit_slice) };

            self.write_exact_poll(init_slice)?;
            n += n_read;
        }

        Ok(())
    }

    fn copy_n_poll(&mut self, reader: &mut R, n_bytes: usize) -> PhonicResult<()> {
        let mut buf = <DefaultBuf<_>>::default();
        self.copy_n_buffered_poll(reader, n_bytes, &mut buf)
    }

    fn copy_all_buffered_poll(
        &mut self,
        reader: &mut R,
        buf: &mut [MaybeUninit<u8>],
    ) -> PhonicResult<()> {
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

            let uninit_slice = &buf[..n_read];
            let init_slice = unsafe { transmute::<&[MaybeUninit<u8>], &[u8]>(uninit_slice) };

            match self.write_exact_poll(init_slice) {
                Ok(()) => continue,
                Err(PhonicError::OutOfBounds) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

    fn copy_all_poll(&mut self, reader: &mut R) -> PhonicResult<()> {
        let mut buf = <DefaultBuf<_>>::default();
        self.copy_all_buffered_poll(reader, &mut buf)
    }
}

impl<T: Stream> PollStream for T {}
impl<T: StreamReader> PollStreamReader for T {}
impl<T: StreamWriter> PollStreamWriter for T {}

impl<T, R> PollStreamCopy<R> for T
where
    Self: Sized + PollStreamWriter,
    R: PollStreamReader,
    R::Tag: TryInto<Self::Tag>,
    PhonicError: From<<R::Tag as TryInto<Self::Tag>>::Error>,
{
}

use crate::{CodecTag, StreamSpec};
use phonic_core::PhonicError;
use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

pub trait Stream {
    type Tag: CodecTag;

    fn stream_spec(&self) -> &StreamSpec<Self::Tag>;
}

pub trait IndexedStream: Stream {
    fn pos(&self) -> u64;

    fn pos_blocks(&self) -> u64 {
        self.pos() / self.stream_spec().block_align as u64
    }

    fn pos_duration(&self) -> Duration {
        let seconds = self.pos() as f64 / self.stream_spec().avg_byte_rate as f64;
        Duration::from_secs_f64(seconds)
    }
}

pub trait FiniteStream: Stream {
    fn len(&self) -> u64;

    fn len_blocks(&self) -> u64 {
        self.len() / self.stream_spec().block_align as u64
    }

    fn len_duration(&self) -> Duration {
        let seconds = self.len() as f64 / self.stream_spec().avg_byte_rate as f64;
        Duration::from_secs_f64(seconds)
    }

    fn is_empty(&self) -> bool
    where
        Self: Sized + IndexedStream,
    {
        self.pos() == self.len()
    }

    fn rem(&self) -> u64
    where
        Self: Sized + IndexedStream,
    {
        self.len() - self.pos()
    }

    fn rem_blocks(&self) -> u64
    where
        Self: Sized + IndexedStream,
    {
        self.rem() / self.stream_spec().block_align as u64
    }

    fn rem_duration(&self) -> Duration
    where
        Self: Sized + IndexedStream,
    {
        self.len_duration() - self.pos_duration()
    }
}

const POLL_TO_BUF_RATIO: u32 = 6;

pub trait StreamReader: Stream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, PhonicError>;

    fn read_exact(&mut self, mut buf: &mut [u8], block: bool) -> Result<(), PhonicError> {
        let buf_len = buf.len();
        if buf_len % self.stream_spec().block_align != 0 {
            return Err(PhonicError::SignalMismatch);
        }

        let poll_interval =
            self.stream_spec().avg_byte_rate_duration() * buf_len as u32 / POLL_TO_BUF_RATIO;

        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Err(PhonicError::Interrupted) if block => continue,
                Err(PhonicError::NotReady) if block => {
                    std::thread::sleep(poll_interval);
                    continue;
                }

                Err(e) => return Err(e),
                Ok(n) => buf = &mut buf[n..],
            };
        }

        Ok(())
    }
}

pub trait StreamWriter: Stream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, PhonicError>;
    fn flush(&mut self) -> Result<(), PhonicError>;

    fn write_exact(&mut self, mut buf: &[u8], block: bool) -> Result<(), PhonicError> {
        let buf_len = buf.len();
        if buf_len % self.stream_spec().block_align != 0 {
            return Err(PhonicError::SignalMismatch);
        }

        let poll_interval =
            self.stream_spec().avg_byte_rate_duration() * buf_len as u32 / POLL_TO_BUF_RATIO;

        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Err(PhonicError::Interrupted) if block => continue,
                Err(PhonicError::NotReady) if block => {
                    std::thread::sleep(poll_interval);
                    continue;
                }

                Err(e) => return Err(e),
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
        block: bool,
    ) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: StreamReader,
        R::Tag: TryInto<Self::Tag>,
    {
        let spec = self.stream_spec();
        let buf_len = buf.len();

        if !spec.is_compatible(reader.stream_spec())
            || n_bytes % spec.block_align != 0
            || buf_len < spec.block_align
        {
            return Err(PhonicError::SignalMismatch);
        }

        let poll_interval =
            self.stream_spec().avg_byte_rate_duration() * buf_len as u32 / POLL_TO_BUF_RATIO;

        let mut n = 0;
        while n < n_bytes {
            let len = buf_len.min(n_bytes - n);
            let n_read = match reader.read(&mut buf[..len]) {
                Ok(0) => return Err(PhonicError::OutOfBounds),
                Err(PhonicError::Interrupted) if block => continue,
                Err(PhonicError::NotReady) if block => {
                    std::thread::sleep(poll_interval);
                    continue;
                }

                Err(e) => return Err(e),
                Ok(n) => n,
            };

            self.write_exact(&buf[..n_read], block)?;
            n += n_read;
        }

        Ok(())
    }

    fn copy_n<R>(&mut self, reader: &mut R, n_bytes: usize, block: bool) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: StreamReader,
        R::Tag: TryInto<Self::Tag>,
    {
        let mut buf = [0u8; 4096];
        self.copy_n_buffered(reader, n_bytes, &mut buf, block)
    }

    fn copy_all_buffered<R>(
        &mut self,
        reader: &mut R,
        buf: &mut [u8],
        block: bool,
    ) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: StreamReader,
        R::Tag: TryInto<Self::Tag>,
    {
        let n_bytes = usize::MAX - (usize::MAX % self.stream_spec().block_align);
        match self.copy_n_buffered(reader, n_bytes, buf, block) {
            Err(PhonicError::OutOfBounds) => Ok(()),
            result => result,
        }
    }

    fn copy_all<R>(&mut self, reader: &mut R, block: bool) -> Result<(), PhonicError>
    where
        Self: Sized,
        R: StreamReader,
        R::Tag: TryInto<Self::Tag>,
    {
        let mut buf = [0u8; 4096];
        self.copy_all_buffered(reader, &mut buf, block)
    }
}

pub trait StreamSeeker: Stream {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError>;

    fn set_pos(&mut self, position: u64) -> Result<(), PhonicError>
    where
        Self: Sized + IndexedStream,
    {
        let offset = position as i64 - self.pos() as i64;
        self.seek(offset)
    }

    fn seek_start(&mut self) -> Result<(), PhonicError>
    where
        Self: Sized + IndexedStream,
    {
        self.set_pos(0)
    }

    fn seek_end(&mut self) -> Result<(), PhonicError>
    where
        Self: Sized + IndexedStream + FiniteStream,
    {
        self.set_pos(self.len())
    }
}

impl<T> Stream for T
where
    T: Deref,
    T::Target: Stream,
{
    type Tag = <T::Target as Stream>::Tag;

    fn stream_spec(&self) -> &StreamSpec<Self::Tag> {
        self.deref().stream_spec()
    }
}

impl<T> IndexedStream for T
where
    T: Deref,
    T::Target: IndexedStream,
{
    fn pos(&self) -> u64 {
        self.deref().pos()
    }
}

impl<T> FiniteStream for T
where
    T: Deref,
    T::Target: FiniteStream,
{
    fn len(&self) -> u64 {
        self.deref().len()
    }
}

impl<T> StreamReader for T
where
    T: DerefMut,
    T::Target: StreamReader,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, PhonicError> {
        self.deref_mut().read(buf)
    }
}

impl<T> StreamWriter for T
where
    T: DerefMut,
    T::Target: StreamWriter,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, PhonicError> {
        self.deref_mut().write(buf)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.deref_mut().flush()
    }
}

impl<T> StreamSeeker for T
where
    T: DerefMut,
    T::Target: StreamSeeker,
{
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        self.deref_mut().seek(offset)
    }
}

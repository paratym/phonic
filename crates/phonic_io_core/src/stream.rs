use crate::{CodecTag, StreamSpec};
use phonic_signal::PhonicResult;
use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut, Neg},
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

pub trait StreamReader: Stream {
    fn read(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize>;
}

pub trait StreamWriter: Stream {
    fn write(&mut self, buf: &[u8]) -> PhonicResult<usize>;
    fn flush(&mut self) -> PhonicResult<()>;
}

pub trait StreamSeeker: Stream {
    fn seek(&mut self, offset: i64) -> PhonicResult<()>;

    fn set_pos(&mut self, pos: u64) -> PhonicResult<()>
    where
        Self: Sized + IndexedStream,
    {
        let current_pos = self.pos();
        let offset = if pos >= current_pos {
            (pos - current_pos) as i64
        } else {
            ((current_pos - pos) as i64).neg()
        };

        self.seek(offset)
    }

    fn seek_start(&mut self) -> PhonicResult<()>
    where
        Self: Sized + IndexedStream,
    {
        self.set_pos(0)
    }

    fn seek_end(&mut self) -> PhonicResult<()>
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
    fn read(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize> {
        self.deref_mut().read(buf)
    }
}

impl<T> StreamWriter for T
where
    T: DerefMut,
    T::Target: StreamWriter,
{
    fn write(&mut self, buf: &[u8]) -> PhonicResult<usize> {
        self.deref_mut().write(buf)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.deref_mut().flush()
    }
}

impl<T> StreamSeeker for T
where
    T: DerefMut,
    T::Target: StreamSeeker,
{
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        self.deref_mut().seek(offset)
    }
}

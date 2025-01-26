use crate::{
    block_on_stream, BlockingStream, FiniteStream, IndexedStream, Stream, StreamReader,
    StreamWriter,
};
use phonic_signal::{utils::slice_as_init_mut, PhonicError, PhonicResult};
use std::mem::MaybeUninit;

pub trait StreamExt: Stream {
    fn rem(&self) -> u64
    where
        Self: IndexedStream + FiniteStream,
    {
        self.len() - self.pos()
    }

    fn is_empty(&self) -> bool
    where
        Self: IndexedStream + FiniteStream,
    {
        self.len() == 0
    }

    fn is_exhausted(&self) -> bool
    where
        Self: IndexedStream + FiniteStream,
    {
        self.pos() == self.len()
    }

    fn read_init<'a>(&mut self, buf: &'a mut [MaybeUninit<u8>]) -> PhonicResult<&'a mut [u8]>
    where
        Self: StreamReader,
    {
        let n_bytes = self.read(buf)?;
        let uninit_slice = &mut buf[..n_bytes];
        let init_slice = unsafe { slice_as_init_mut(uninit_slice) };

        Ok(init_slice)
    }

    fn read_blocking(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize>
    where
        Self: BlockingStream + StreamReader,
    {
        block_on_stream!(self, self.read(buf))
    }

    fn read_init_blocking<'a>(
        &mut self,
        buf: &'a mut [MaybeUninit<u8>],
    ) -> PhonicResult<&'a mut [u8]>
    where
        Self: BlockingStream + StreamReader,
    {
        block_on_stream!(self, self.read_init(buf), result => result.map(|init| unsafe {
            std::slice::from_raw_parts_mut(init.as_mut_ptr(), init.len())
        }))
    }

    fn read_exact(&mut self, mut buf: &mut [MaybeUninit<u8>]) -> PhonicResult<()>
    where
        Self: BlockingStream + StreamReader,
    {
        if buf.len() % self.stream_spec().block_align != 0 {
            return Err(PhonicError::invalid_input());
        }

        while !buf.is_empty() {
            match self.read(buf) {
                Ok(0) => return Err(PhonicError::out_of_bounds()),
                Ok(n) => buf = &mut buf[n..],
                Err(PhonicError::Interrupted { .. }) => continue,
                Err(PhonicError::NotReady { .. }) => self.block(),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    fn read_exact_init<'a>(&mut self, buf: &'a mut [MaybeUninit<u8>]) -> PhonicResult<&'a [u8]>
    where
        Self: BlockingStream + StreamReader,
    {
        self.read_exact(buf)?;
        Ok(unsafe { slice_as_init_mut(buf) })
    }

    fn write_blocking(&mut self, buf: &[u8]) -> PhonicResult<usize>
    where
        Self: BlockingStream + StreamWriter,
    {
        block_on_stream!(self, self.write(buf))
    }

    fn flush_blocking(&mut self) -> PhonicResult<()>
    where
        Self: BlockingStream + StreamWriter,
    {
        block_on_stream!(self, self.flush())
    }

    fn write_exact(&mut self, mut buf: &[u8]) -> PhonicResult<()>
    where
        Self: BlockingStream + StreamWriter,
    {
        if buf.len() % self.stream_spec().block_align != 0 {
            return Err(PhonicError::invalid_input());
        }

        while !buf.is_empty() {
            match self.write_blocking(buf) {
                Ok(0) => return Err(PhonicError::out_of_bounds()),
                Ok(n) => buf = &buf[n..],
                Err(PhonicError::Interrupted { .. }) => continue,
                Err(PhonicError::NotReady { .. }) => self.block(),
                Err(e) => return Err(e),
            };
        }

        Ok(())
    }
}

impl<T: Stream> StreamExt for T {}

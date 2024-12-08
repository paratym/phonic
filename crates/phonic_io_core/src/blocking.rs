use crate::{FormatReader, FormatWriter, StreamReader, StreamWriter};
use phonic_signal::PhonicResult;
use std::{mem::MaybeUninit, ops::DerefMut};

pub trait BlockingFormatReader: FormatReader {
    fn read_blocking(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<(usize, usize)>;
}

pub trait BlockingFormatWriter: FormatWriter {
    fn write_blocking(&mut self, stream: usize, buf: &[u8]) -> PhonicResult<usize>;
    fn flush_blocking(&mut self) -> PhonicResult<()>;
    fn finalize_blocking(&mut self) -> PhonicResult<()>;
}

pub trait BlockingStreamReader: StreamReader {
    fn read_blocking(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize>;
}

pub trait BlockingStreamWriter: StreamWriter {
    fn write_blocking(&mut self, buf: &[u8]) -> PhonicResult<usize>;
    fn flush_blocking(&mut self) -> PhonicResult<()>;
}

impl<T> BlockingFormatReader for T
where
    T: DerefMut,
    T::Target: BlockingFormatReader,
{
    fn read_blocking(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<(usize, usize)> {
        self.deref_mut().read_blocking(buf)
    }
}

impl<T> BlockingFormatWriter for T
where
    T: DerefMut,
    T::Target: BlockingFormatWriter,
{
    fn write_blocking(&mut self, stream: usize, buf: &[u8]) -> PhonicResult<usize> {
        self.deref_mut().write_blocking(stream, buf)
    }

    fn flush_blocking(&mut self) -> PhonicResult<()> {
        self.deref_mut().flush_blocking()
    }

    fn finalize_blocking(&mut self) -> PhonicResult<()> {
        self.deref_mut().finalize_blocking()
    }
}

impl<T> BlockingStreamReader for T
where
    T: DerefMut,
    T::Target: BlockingStreamReader,
{
    fn read_blocking(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize> {
        self.deref_mut().read_blocking(buf)
    }
}

impl<T> BlockingStreamWriter for T
where
    T: DerefMut,
    T::Target: BlockingStreamWriter,
{
    fn write_blocking(&mut self, buf: &[u8]) -> PhonicResult<usize> {
        self.deref_mut().write_blocking(buf)
    }

    fn flush_blocking(&mut self) -> PhonicResult<()> {
        self.deref_mut().flush_blocking()
    }
}

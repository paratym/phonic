use crate::{
    delegate_format, delegate_stream, BlockingFormatReader, BlockingFormatWriter,
    BlockingStreamReader, BlockingStreamWriter, FormatReader, FormatWriter, StreamReader,
    StreamWriter,
};
use phonic_signal::{utils::Poll, PhonicError, PhonicResult};
use std::mem::MaybeUninit;

pub struct PollIo<T>(pub T);

delegate_stream! {
    delegate<T> * + !Blocking for PollIo<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

delegate_format! {
    delegate<T> * + !Blocking for PollIo<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

macro_rules! poll {
    ($func:expr) => {
        loop {
            match $func {
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => Poll::<()>::poll_interval(),
                result => return result,
            }
        }
    };
}

impl<T: StreamReader> BlockingStreamReader for PollIo<T> {
    fn read_blocking(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize> {
        poll!(self.0.read(buf))
    }
}

impl<T: StreamWriter> BlockingStreamWriter for PollIo<T> {
    fn write_blocking(&mut self, buf: &[u8]) -> PhonicResult<usize> {
        poll!(self.0.write(buf))
    }

    fn flush_blocking(&mut self) -> phonic_signal::PhonicResult<()> {
        poll!(self.0.flush())
    }
}

impl<T: FormatReader> BlockingFormatReader for PollIo<T> {
    fn read_blocking(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<(usize, usize)> {
        poll!(self.0.read(buf))
    }
}

impl<T: FormatWriter> BlockingFormatWriter for PollIo<T> {
    fn write_blocking(&mut self, stream: usize, buf: &[u8]) -> PhonicResult<usize> {
        poll!(self.0.write(stream, buf))
    }

    fn flush_blocking(&mut self) -> PhonicResult<()> {
        poll!(self.0.flush())
    }

    fn finalize_blocking(&mut self) -> phonic_signal::PhonicResult<()> {
        poll!(self.0.finalize())
    }
}

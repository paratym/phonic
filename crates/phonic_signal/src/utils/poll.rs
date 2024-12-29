use crate::{
    delegate_signal, BlockingSignalReader, BlockingSignalWriter, PhonicError, PhonicResult,
    SignalReader, SignalWriter,
};
use std::{mem::MaybeUninit, time::Duration};

pub struct Poll<T>(pub T);

impl<T> Poll<T> {
    pub fn poll_interval() {
        std::thread::sleep(Duration::from_millis(10))

        // TODO
        // https://doc.rust-lang.org/std/hint/fn.spin_loop.html
        // https://doc.rust-lang.org/std/thread/fn.yield_now.html
    }
}

delegate_signal! {
    delegate<T> * + !Blocking for Poll<T> {
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
                Err(PhonicError::NotReady) => Self::poll_interval(),
                result => return result,
            }
        }
    };
}

impl<T> BlockingSignalReader for Poll<T>
where
    Self: SignalReader,
{
    fn read_blocking(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        poll!(self.read(buf))
    }
}

impl<T> BlockingSignalWriter for Poll<T>
where
    Self: SignalWriter,
{
    fn write_blocking(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        poll!(self.write(buf))
    }

    fn flush_blocking(&mut self) -> PhonicResult<()> {
        poll!(self.flush())
    }
}

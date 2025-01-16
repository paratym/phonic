use crate::{delegate_signal, BlockingSignal, Signal};
use std::time::Duration;

#[repr(transparent)]
pub struct Poll<T>(pub T);

#[macro_export]
macro_rules! block_on_signal {
    ($self:expr, $func:expr, $result:pat => $return:expr) => {
        loop {
            match $func {
                ::std::result::Result::Err($crate::PhonicError::Interrupted) => continue,
                ::std::result::Result::Err($crate::PhonicError::NotReady) => $crate::BlockingSignal::block($self),
                $result => break $return,
            }
        }
    };
    ($self:expr, $func:expr) => {
        $crate::block_on_signal!($self, $func, result => result)
    }
}

delegate_signal! {
    impl<T> * + !BlockingSignal for Poll<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T> Poll<T> {
    pub fn interval() {
        std::thread::sleep(Duration::from_millis(10))

        // TODO
        // https://doc.rust-lang.org/std/hint/fn.spin_loop.html
        // https://doc.rust-lang.org/std/thread/fn.yield_now.html
    }
}

impl<T: Signal> BlockingSignal for Poll<T> {
    fn block(&self) {
        Self::interval()
    }
}

use crate::{delegate_signal, BlockingSignal, Signal};
use std::time::Duration;

pub struct Poll<T>(pub T);

delegate_signal! {
    impl<T> * + !BlockingSignal for Poll<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Signal> BlockingSignal for Poll<T> {
    fn block(&self) {
        std::thread::sleep(Duration::from_millis(10))

        // TODO
        // https://doc.rust-lang.org/std/hint/fn.spin_loop.html
        // https://doc.rust-lang.org/std/thread/fn.yield_now.html
    }
}

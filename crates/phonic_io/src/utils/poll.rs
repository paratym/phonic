use crate::{delegate_format, delegate_stream, BlockingFormat, BlockingStream, Format, Stream};
use phonic_signal::utils::Poll;

#[repr(transparent)]
pub struct PollIo<T>(pub T);

delegate_stream! {
    impl<T> * + !BlockingStream for PollIo<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

delegate_format! {
    impl<T> * + !BlockingFormat for PollIo<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

#[macro_export]
macro_rules! block_on_format {
    ($self:expr, $func:expr, $result:pat => $return:expr) => {
        loop {
            match $func {
                ::std::result::Result::Err(::phonic_signal::PhonicError::Interrupted) => continue,
                ::std::result::Result::Err(::phonic_signal::PhonicError::NotReady) => {
                    $crate::BlockingFormat::block($self)
                }

                $result => break $return,
            }
        }
    };
    ($self:expr, $func:expr) => {
        $crate::block_on_format!($self, $func, result => result)
    }
}

#[macro_export]
macro_rules! block_on_stream {
    ($self:expr, $func:expr, $result:pat => $return:expr) => {
        loop {
            match $func {
                ::std::result::Result::Err(::phonic_signal::PhonicError::Interrupted) => continue,
                ::std::result::Result::Err(::phonic_signal::PhonicError::NotReady) => {
                    $crate::BlockingStream::block($self)
                }

                $result => break $return,
            }
        }
    };
    ($self:expr, $func:expr) => {
        $crate::block_on_stream!($self, $func, result => result)
    }
}

impl<T: Format> BlockingFormat for PollIo<T> {
    fn block(&self) {
        Poll::<()>::interval()
    }
}

impl<T: Stream> BlockingStream for PollIo<T> {
    fn block(&self) {
        Poll::<()>::interval()
    }
}

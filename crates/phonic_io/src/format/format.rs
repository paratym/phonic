use phonic_macro::delegate_group;
use std::ops::{Deref, DerefMut};

delegate_group! {
    mod as crate;

    pub trait Format {
        type Tag: crate::FormatTag;

        fn format(&self) -> Self::Tag;
        fn streams(&self) -> &[crate::StreamSpec<<Self::Tag as crate::FormatTag>::Codec>];

        fn current_stream(&self) -> usize;

        fn primary_stream(&self) -> Option<usize> {
            match self.streams() {
                [_] => Some(0),
                [..] => None,
            }
        }
    }

    pub trait BlockingFormat: crate::Format {
        fn block(&self);
    }

    pub trait IndexedFormat: crate::Format {
        fn pos(&self) -> u64;
        fn stream_pos(&self, stream: usize) -> u64;
    }

    pub trait FiniteFormat: crate::Format {
        fn len(&self) -> u64;
        fn stream_len(&self, stream: usize) -> u64;
    }

    #[subgroup(Mut)]
    pub trait FormatReader: crate::Format {
        fn read(
            &mut self,
            buf: &mut [std::mem::MaybeUninit<u8>]
        ) -> phonic_signal::PhonicResult<(usize, usize)>;
    }

    #[subgroup(Mut)]
    pub trait FormatWriter: crate::Format {
        fn write(&mut self, stream: usize, buf: &[u8]) -> phonic_signal::PhonicResult<usize>;
        fn flush(&mut self) -> phonic_signal::PhonicResult<()>;
        fn finalize(&mut self) -> phonic_signal::PhonicResult<()>;
    }


    pub trait FormatSeeker: crate::Format {
        fn seek(&mut self, stream: usize, offset: i64) -> phonic_signal::PhonicResult<()>;
    }
}

delegate_format! {
    impl<T> * for T {
        Self as T::Target;

        &self => self.deref()
        where T: Deref;

        &mut self => self.deref_mut()
        where T: DerefMut;
    }
}

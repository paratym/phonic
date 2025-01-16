use phonic_macro::delegate_group;
use std::ops::{Deref, DerefMut};

delegate_group! {
    mod as crate;

    pub trait Stream {
        type Tag: crate::CodecTag;

        fn stream_spec(&self) -> &crate::StreamSpec<Self::Tag>;
    }

    pub trait BlockingStream: Stream {
        fn block(&self);
    }

    pub trait IndexedStream: Stream {
        /// retuns the number of bytes between the start and current position of the stream
        fn pos(&self) -> u64;

    }

    pub trait FiniteStream: Stream {
        /// returns the number of bytes between the start and end of the stream
        fn len(&self) -> u64;

    }

    #[subgroup(Mut, Read)]
    pub trait StreamReader: Stream {
        fn read(
            &mut self,
            buf: &mut [std::mem::MaybeUninit<u8>]
        ) -> phonic_signal::PhonicResult<usize>;
    }

    #[subgroup(Mut, Read, Buffered)]
    pub trait BufferedStreamReader {
        fn fill(&mut self) -> phonic_signal::PhonicResult<&[u8]>;
        fn buffer(&self) -> Option<&[u8]>;
        fn consume(&mut self, n_bytes: usize);
    }


    #[subgroup(Mut, Write)]
    pub trait StreamWriter: Stream {
        fn write(&mut self, buf: &[u8]) -> phonic_signal::PhonicResult<usize>;
        fn flush(&mut self) -> phonic_signal::PhonicResult<()>;
    }

    #[subgroup(Mut, Write, Buffered)]
    pub trait BufferedStreamWriter {
        fn buffer_mut(&mut self) -> phonic_signal::PhonicResult<&mut [u8]>;
        fn commit(&mut self, n_bytes: usize);
    }

    #[subgroup(Mut)]
    pub trait StreamSeeker: Stream {
        fn seek(&mut self, offset: i64) -> phonic_signal::PhonicResult<()>;
    }
}

delegate_stream! {
    impl<T> * for T {
        Self as T::Target;

        &self => self.deref()
        where T: Deref;

        &mut self => self.deref_mut()
        where T: DerefMut;
    }
}

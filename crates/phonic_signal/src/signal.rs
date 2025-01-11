use phonic_macro::delegate_group;
use std::ops::{Deref, DerefMut};

delegate_group! {
    mod as crate;

    pub trait Signal {
        type Sample: crate::Sample;

        fn spec(&self) -> &crate::SignalSpec;
    }

    pub trait BlockingSignal: crate::Signal {
        fn block(&self);
    }

    pub trait IndexedSignal: crate::Signal {
        fn pos(&self) -> u64;
    }

    pub trait FiniteSignal: crate::Signal {
        fn len(&self) -> u64;
    }

    #[subgroup(Mut, Read)]
    pub trait SignalReader: crate::Signal {
        /// reads samples from this signal into the given buffer.
        /// returns the number of interleaved samples read.
        fn read(
            &mut self,
            buf: &mut [std::mem::MaybeUninit<Self::Sample>]
        ) -> crate::PhonicResult<usize>;
    }

    #[subgroup(Mut, Read, Buffered)]
    pub trait BufferedSignalReader: crate::SignalReader {
        /// Ensures there are samples available in this signal's inner buffer and returns a
        /// reference to them. On "pull-based" signal chains the samples are read from the next
        /// source. On "push-based" signal chains `Err(PhonicError::NotReady)` is returned until
        /// there are samples available.
        fn fill(&mut self) -> crate::PhonicResult<&[Self::Sample]>;

        /// Returns a reference to this signal's inner buffer, or `None` if this signal is
        /// exhausted. To handle an empty buffer see `BufferedSignalReader::fill`.
        fn buffer(&self) -> Option<&[Self::Sample]>;

        /// Moves the read/write head forward by `n_samples` and frees the consumed section of
        /// this signal's inner buffer for reuse.
        ///
        /// # Panics
        /// panics if `n_samples` is greater than the length of the available inner buffer.
        fn consume(&mut self, n_samples: usize);
    }

    #[subgroup(Mut, Write)]
    pub trait SignalWriter: crate::Signal {
        /// writes samples from the given buffer to this signal.
        /// returns the number of interleaved samples written.
        fn write(&mut self, buf: &[Self::Sample]) -> crate::PhonicResult<usize>;

        /// Ensures all samples in the signal chain have been written to the innermost
        /// destination. On "push-based" signal chains the samples are recursively written to
        /// the next destination. On "pull-based" signal chains `Err(PhonicError::NotReady)` is
        /// returned until there are no samples left.
        fn flush(&mut self) -> crate::PhonicResult<()>;
    }

    #[subgroup(Mut, Write, Buffered)]
    pub trait BufferedSignalWriter: crate::SignalWriter {
        /// Returns a mutable reference to this signal's inner buffer, or `None` if this signal
        /// is exhausted. To handle an empty buffer, see `SignalWriter::flush`.
        fn buffer_mut(&mut self) -> Option<&mut [std::mem::MaybeUninit<Self::Sample>]>;

        /// Moves the read/write head forward by `n_samples` and marks the committed section of this
        /// signal's inner buffer to be written to the underlying writer. To ensure the marked
        /// samples have been written see `SignalWriter::flush`.
        ///
        /// # Panics
        /// panics if `n_samples` is greater than the length of the available inner buffer.
        fn commit(&mut self, n_samples: usize);
    }

    #[subgroup(Mut)]
    pub trait SignalSeeker: crate::Signal {
        fn seek(&mut self, n_frames: i64) -> crate::PhonicResult<()>;
    }
}

delegate_signal! {
    impl<T> * for T {
        Self as T::Target;

        &self => self.deref()
        where T: Deref;

        &mut self => self.deref_mut()
        where T: DerefMut;
    }
}

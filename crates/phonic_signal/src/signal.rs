use phonic_macro::delegate_group;
use std::ops::{Deref, DerefMut};

delegate_group! {
    mod as crate;

    pub trait Signal {
        /// The type of sample that can be read from/written to this signal.
        type Sample: crate::Sample;

        /// Returns a set of parameters that determines how the samples that are read from/written
        /// to this signal should be interpreted.
        fn spec(&self) -> &crate::SignalSpec;
    }

    pub trait BlockingSignal: crate::Signal {
        /// Blocks the current thread for an unspecified period of time or until an unknown event
        /// occurs. Generally this method is used inside of a loop to handle
        /// `PhonicError::NotReady`
        fn block(&self);
    }

    pub trait IndexedSignal: crate::Signal {
        /// Returns the number of frames between the start and the read/write head of this signal.
        fn pos(&self) -> u64;
    }

    pub trait FiniteSignal: crate::Signal {
        /// Returns the number of frames between the start and the end of this signal.
        fn len(&self) -> u64;
    }

    #[subgroup(Mut, Read)]
    pub trait SignalReader: crate::Signal {
        /// Reads samples from this signal into the given buffer and returns the number of
        /// interleaved samples read.
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

        /// Moves this signal's read/write head forward by `n_samples` and frees the consumed
        /// section of its inner buffer for reuse.
        ///
        /// # Panics
        /// panics if `n_samples` is greater than the length of the available inner buffer.
        fn consume(&mut self, n_samples: usize);
    }

    #[subgroup(Mut, Write)]
    pub trait SignalWriter: crate::Signal {
        /// Writes samples from the given buffer to this signal and returns the number of
        /// interleaved samples written.
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

        /// Moves this signal's read/write head forward by `n_samples` and marks the committed
        /// section of its inner buffer to be written to the underlying writer. To ensure the
        /// marked samples have been written see `SignalWriter::flush`.
        ///
        /// # Panics
        /// panics if `n_samples` is greater than the length of the available inner buffer.
        fn commit(&mut self, n_samples: usize);
    }

    #[subgroup(Mut)]
    pub trait SignalSeeker: crate::Signal {
        /// Moves this signal's read/write head by `n_frames`.
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

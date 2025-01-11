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

        fn current_stream_spec(&self) -> &crate::StreamSpec<<Self::Tag as crate::FormatTag>::Codec> {
            let i = self.current_stream();
            &self.streams()[i]
        }

        fn primary_stream_spec(&self) -> Option<&crate::StreamSpec<<Self::Tag as crate::FormatTag>::Codec>> {
            self.primary_stream().and_then(|i| self.streams().get(i))
        }
    }

    pub trait IndexedFormat: Format {
        fn pos(&self) -> u64;
        fn stream_pos(&self, stream: usize) -> u64;
    }

    pub trait FiniteFormat: Format {
        fn len(&self) -> u64;
        fn stream_len(&self, stream: usize) -> u64;

        fn is_empty(&self) -> bool
        where
            Self: Sized + IndexedFormat,
        {
            self.pos() == self.len()
        }
    }

    #[subgroup(Mut, Read)]
    pub trait FormatReader: Format {
        fn read(&mut self, buf: &mut [std::mem::MaybeUninit<u8>]) -> phonic_signal::PhonicResult<(usize, usize)>;

        fn read_init<'a>(
            &mut self,
            buf: &'a mut [std::mem::MaybeUninit<u8>],
        ) -> phonic_signal::PhonicResult<(usize, &'a mut [u8])> {
            let (stream_i, n_bytes) = self.read(buf)?;
            let uninit_slice = &mut buf[..n_bytes];
            let init_slice = unsafe { phonic_signal::utils::slice_as_init_mut(uninit_slice) };

            Ok((stream_i, init_slice))
        }
    }

    #[subgroup(Mut, Read, Blocking)]
    pub trait BlockingFormatReader: FormatReader {
        fn read_blocking(&mut self, buf: &mut [std::mem::MaybeUninit<u8>]) -> phonic_signal::PhonicResult<(usize, usize)>;
    }

    #[subgroup(Mut, Write)]
    pub trait FormatWriter: Format {
        fn write(&mut self, stream: usize, buf: &[u8]) -> phonic_signal::PhonicResult<usize>;
        fn flush(&mut self) -> phonic_signal::PhonicResult<()>;
        fn finalize(&mut self) -> phonic_signal::PhonicResult<()>;
    }

    #[subgroup(Mut, Write, Blocking)]
    pub trait BlockingFormatWriter: FormatWriter {
        fn write_blocking(&mut self, stream: usize, buf: &[u8]) -> phonic_signal::PhonicResult<usize>;
        fn flush_blocking(&mut self) -> phonic_signal::PhonicResult<()>;
        fn finalize_blocking(&mut self) -> phonic_signal::PhonicResult<()>;
    }

    pub trait FormatSeeker: Format {
        fn seek(&mut self, stream: usize, offset: i64) -> phonic_signal::PhonicResult<()>;

        fn set_pos(&mut self, stream: usize, pos: u64) -> Result<(), phonic_signal::PhonicError>
        where
            Self: Sized + IndexedFormat,
        {
            let current_pos = self.stream_pos(stream);
            let offset = if pos >= current_pos {
                (pos - current_pos) as i64
            } else {
                -((current_pos - pos) as i64)
            };

            self.seek(stream, offset)
        }
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

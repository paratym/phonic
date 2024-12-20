use phonic_macro::delegate_group;

delegate_group! {
    #![mod_path(crate)]

    pub trait Stream {
        type Tag: crate::CodecTag;

        fn stream_spec(&self) -> &crate::StreamSpec<Self::Tag>;
    }

    pub trait IndexedStream: Stream {
        /// retuns the number of bytes between the start and current position of the stream
        fn pos(&self) -> u64;

        fn pos_duration<D: crate::StreamDuration>(&self) -> D
        where
            Self: Sized
        {
            use crate::IntoStreamDuration;
            crate::NBytes::from(self.pos()).into_stream_duration(self.stream_spec())
        }
    }

    pub trait FiniteStream: Stream {
        /// returns the number of bytes between the start and end of the stream
        fn len(&self) -> u64;

        fn len_duration<D: crate::StreamDuration>(&self) -> D
        where
            Self: Sized
        {
            use crate::IntoStreamDuration;
            crate::NBytes::from(self.len()).into_stream_duration(self.stream_spec())
        }

        fn is_empty(&self) -> bool
        where
            Self: Sized + IndexedStream,
        {
            self.pos() == self.len()
        }

        fn rem(&self) -> u64
        where
            Self: Sized + IndexedStream,
        {
            self.len() - self.pos()
        }

        fn rem_duration(&self) -> std::time::Duration
        where
            Self: Sized + IndexedStream,
        {
            use crate::IntoStreamDuration;
            crate::NBytes::from(self.rem()).into_stream_duration(self.stream_spec())
        }
    }

    #[subgroup(Mut, Read)]
    pub trait StreamReader: Stream {
        fn read(&mut self, buf: &mut [std::mem::MaybeUninit<u8>]) -> phonic_signal::PhonicResult<usize>;

        fn read_init<'a>(&mut self, buf: &'a mut [std::mem::MaybeUninit<u8>]) -> phonic_signal::PhonicResult<&'a mut [u8]> {
            let n_bytes = self.read(buf)?;
            let uninit_slice = &mut buf[..n_bytes];
            let init_slice = unsafe { phonic_signal::utils::slice_as_init_mut(uninit_slice) };

            Ok(init_slice)
        }
    }

    #[subgroup(Mut, Read, Blocking)]
    pub trait BlockingStreamReader: StreamReader {
        fn read_blocking(&mut self, buf: &mut [std::mem::MaybeUninit<u8>]) -> phonic_signal::PhonicResult<usize>;
    }

    #[subgroup(Mut, Write)]
    pub trait StreamWriter: Stream {
        fn write(&mut self, buf: &[u8]) -> phonic_signal::PhonicResult<usize>;
        fn flush(&mut self) -> phonic_signal::PhonicResult<()>;
    }

    #[subgroup(Mut, Write, Blocking)]
    pub trait BlockingStreamWriter: StreamWriter {
        fn write_blocking(&mut self, buf: &[u8]) -> phonic_signal::PhonicResult<usize>;
        fn flush_blocking(&mut self) -> phonic_signal::PhonicResult<()>;
    }

    #[subgroup(Mut)]
    pub trait StreamSeeker: Stream {
        fn seek(&mut self, offset: i64) -> phonic_signal::PhonicResult<()>;
    }
}

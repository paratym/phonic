use phonic_macro::delegate_group;
use std::ops::{Deref, DerefMut};

delegate_group! {
    #![mod_path(crate)]

    pub trait Stream {
        type Tag: crate::CodecTag;

    fn stream_spec(&self) -> &crate::StreamSpec<Self::Tag>;
    }

    pub trait IndexedStream: Stream {
        fn pos(&self) -> u64;

        fn pos_blocks(&self) -> u64 {
            self.pos() / self.stream_spec().block_align as u64
        }

        fn pos_duration(&self) -> std::time::Duration {
            let seconds = self.pos() as f64 / self.stream_spec().avg_byte_rate as f64;
            std::time::Duration::from_secs_f64(seconds)
        }
    }

    pub trait FiniteStream: Stream {
        fn len(&self) -> u64;

        fn len_blocks(&self) -> u64 {
            self.len() / self.stream_spec().block_align as u64
        }

        fn len_duration(&self) -> std::time::Duration {
            let seconds = self.len() as f64 / self.stream_spec().avg_byte_rate as f64;
            std::time::Duration::from_secs_f64(seconds)
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

        fn rem_blocks(&self) -> u64
        where
            Self: Sized + IndexedStream,
        {
            self.rem() / self.stream_spec().block_align as u64
        }

        fn rem_duration(&self) -> std::time::Duration
        where
            Self: Sized + IndexedStream,
        {
            self.len_duration() - self.pos_duration()
        }
    }

    pub trait StreamReader: Stream {
        fn read(&mut self, buf: &mut [std::mem::MaybeUninit<u8>]) -> phonic_signal::PhonicResult<usize>;

        fn read_init<'a>(&mut self, buf: &'a mut [std::mem::MaybeUninit<u8>]) -> phonic_signal::PhonicResult<&'a mut [u8]> {
            let n_bytes = self.read(buf)?;
            let uninit_slice = &mut buf[..n_bytes];
            let init_slice = unsafe { phonic_signal::utils::slice_as_init_mut(uninit_slice) };

            Ok(init_slice)
        }
    }

    pub trait StreamWriter: Stream {
        fn write(&mut self, buf: &[u8]) -> phonic_signal::PhonicResult<usize>;
        fn flush(&mut self) -> phonic_signal::PhonicResult<()>;
    }

    pub trait StreamSeeker: Stream {
        fn seek(&mut self, offset: i64) -> phonic_signal::PhonicResult<()>;

        fn set_pos(&mut self, pos: u64) -> phonic_signal::PhonicResult<()>
        where
            Self: Sized + IndexedStream,
        {
            let current_pos = self.pos();
            let offset = if pos >= current_pos {
                (pos - current_pos) as i64
            } else {
                -((current_pos - pos) as i64)
            };

            self.seek(offset)
        }

        fn seek_start(&mut self) -> phonic_signal::PhonicResult<()>
        where
            Self: Sized + IndexedStream,
        {
            self.set_pos(0)
        }

        fn seek_end(&mut self) -> phonic_signal::PhonicResult<()>
        where
            Self: Sized + IndexedStream + FiniteStream,
        {
            self.set_pos(self.len())
        }
    }
}

delegate_stream! {
    delegate<T> * for T {
        Self as T::Target;

        &self => self.deref()
        where T: Deref;

        &mut self => self.deref_mut()
        where T: DerefMut;
    }
}

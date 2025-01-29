use crate::{
    delegate_format, delegate_stream, FiniteFormat, FiniteStream, Format, FormatReader,
    FormatSeeker, FormatWriter, Stream, StreamReader, StreamSeeker, StreamWriter,
};
use phonic_signal::{
    delegate_signal, FiniteSignal, PhonicError, PhonicResult, Signal, SignalReader, SignalSeeker,
    SignalWriter,
};
use std::{io, mem::MaybeUninit};

#[repr(transparent)]
pub struct Infinite<T>(pub T);

#[repr(transparent)]
pub struct UnReadable<T>(pub T);

#[repr(transparent)]
pub struct UnWriteable<T>(pub T);

#[repr(transparent)]
pub struct UnSeekable<T>(pub T);

delegate_format! {
    impl<T> * + !FiniteFormat for Infinite<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Format> FiniteFormat for Infinite<T> {
    fn len(&self) -> u64 {
        0
    }

    fn stream_len(&self, _stream: usize) -> u64 {
        0
    }
}

delegate_stream! {
    impl<T> * + !FiniteStream for Infinite<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Stream> FiniteStream for Infinite<T> {
    fn len(&self) -> u64 {
        0
    }
}

delegate_signal! {
    impl<T> * + !FiniteSignal for Infinite<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Signal> FiniteSignal for Infinite<T> {
    fn len(&self) -> u64 {
        0
    }
}

impl<T: io::Read> io::Read for UnReadable<T> {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "reading is not supported by the reciever",
        ))
    }
}

delegate_format! {
    impl<T> * + !FormatReader for UnReadable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Format> FormatReader for UnReadable<T> {
    fn read(&mut self, _buf: &mut [MaybeUninit<u8>]) -> PhonicResult<(usize, usize)> {
        Err(PhonicError::unsupported())
    }
}

delegate_stream! {
    impl<T> * + !Read for UnReadable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Stream> StreamReader for UnReadable<T> {
    fn read(&mut self, _buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize> {
        Err(PhonicError::unsupported())
    }
}

delegate_signal! {
    impl<T> * + !Read for UnReadable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Signal> SignalReader for UnReadable<T> {
    fn read(&mut self, _buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        Err(PhonicError::unsupported())
    }
}

impl<T: io::Write> io::Write for UnWriteable<T> {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "writing is not supported by the reciever",
        ))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

delegate_format! {
    impl<T> * + !FormatWriter for UnWriteable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Format> FormatWriter for UnWriteable<T> {
    fn write(&mut self, _stream: usize, _buf: &[u8]) -> PhonicResult<usize> {
        Err(PhonicError::unsupported())
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Ok(())
    }

    fn finalize(&mut self) -> PhonicResult<()> {
        Ok(())
    }
}

delegate_stream! {
    impl<T> * + !Write for UnWriteable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Stream> StreamWriter for UnWriteable<T> {
    fn write(&mut self, _buf: &[u8]) -> PhonicResult<usize> {
        Err(PhonicError::unsupported())
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Ok(())
    }
}

delegate_signal! {
    impl<T> * + !Write for UnWriteable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Signal> SignalWriter for UnWriteable<T> {
    fn write(&mut self, _buf: &[Self::Sample]) -> PhonicResult<usize> {
        Err(PhonicError::unsupported())
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Ok(())
    }
}

impl<T: io::Seek> io::Seek for UnSeekable<T> {
    fn seek(&mut self, _pos: io::SeekFrom) -> io::Result<u64> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "seeking is not supported by the reciever",
        ))
    }
}

delegate_format! {
    impl<T> * + !FormatSeeker for UnSeekable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Format> FormatSeeker for UnSeekable<T> {
    fn seek(&mut self, _stream: usize, _offset: i64) -> PhonicResult<()> {
        Err(PhonicError::unsupported())
    }
}

delegate_stream! {
    impl<T> * + !StreamSeeker for UnSeekable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Stream> StreamSeeker for UnSeekable<T> {
    fn seek(&mut self, _offset: i64) -> PhonicResult<()> {
        Err(PhonicError::unsupported())
    }
}

delegate_signal! {
    impl<T> * + !SignalSeeker for UnSeekable<T> {
        Self as T;

        &self => &self.0;
        &mut self => &mut self.0;
    }
}

impl<T: Signal> SignalSeeker for UnSeekable<T> {
    fn seek(&mut self, _n_frames: i64) -> PhonicResult<()> {
        Err(PhonicError::unsupported())
    }
}

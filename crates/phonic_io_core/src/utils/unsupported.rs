use crate::{
    FiniteStream, Format, FormatReader, FormatSeeker, FormatTag, FormatWriter, IndexedStream,
    Stream, StreamReader, StreamSeeker, StreamSpec, StreamWriter,
};
use phonic_signal::{
    FiniteSignal, PhonicError, PhonicResult, Signal, SignalReader, SignalSeeker, SignalSpec,
    SignalWriter,
};

pub struct Infinite<T>(pub T);
pub struct UnReadable<T>(pub T);
pub struct UnWriteable<T>(pub T);
pub struct UnSeekable<T>(pub T);

pub trait Unsupported: Sized {
    fn infinite(self) -> Infinite<Self> {
        Infinite(self)
    }

    fn un_readable(self) -> UnReadable<Self> {
        UnReadable(self)
    }

    fn un_writeable(self) -> UnWriteable<Self> {
        UnWriteable(self)
    }

    fn un_seekable(self) -> UnSeekable<Self> {
        UnSeekable(self)
    }
}

impl<T> Unsupported for T {}

macro_rules! impl_format {
    ($name:ident) => {
        impl<T: Format> Format for $name<T> {
            type Tag = T::Tag;

            fn format(&self) -> Self::Tag {
                self.0.format()
            }

            fn streams(&self) -> &[StreamSpec<<Self::Tag as FormatTag>::Codec>] {
                self.0.streams()
            }

            fn current_stream(&self) -> usize {
                self.0.current_stream()
            }
        }
    };
}

macro_rules! impl_format_read {
    ($name:ident) => {
        impl<T: FormatReader> FormatReader for $name<T> {
            fn read(&mut self, buf: &mut [u8]) -> PhonicResult<(usize, usize)> {
                self.0.read(buf)
            }
        }
    };
}

macro_rules! impl_format_write {
    ($name:ident) => {
        impl<T: FormatWriter> FormatWriter for $name<T> {
            fn write(&mut self, stream: usize, buf: &[u8]) -> PhonicResult<usize> {
                self.0.write(stream, buf)
            }

            fn flush(&mut self) -> PhonicResult<()> {
                self.0.flush()
            }

            fn finalize(&mut self) -> PhonicResult<()> {
                self.0.finalize()
            }
        }
    };
}

macro_rules! impl_format_seek {
    ($name:ident) => {
        impl<T: FormatSeeker> FormatSeeker for $name<T> {
            fn seek(&mut self, stream: usize, offset: i64) -> PhonicResult<()> {
                self.0.seek(stream, offset)
            }
        }
    };
}

impl_format!(UnReadable);
impl_format!(UnWriteable);
impl_format!(UnSeekable);

impl_format_read!(UnWriteable);
impl_format_read!(UnSeekable);

impl_format_write!(UnReadable);
impl_format_write!(UnSeekable);

impl_format_seek!(UnReadable);
impl_format_seek!(UnWriteable);

impl<T: Format> FormatReader for UnReadable<T> {
    fn read(&mut self, _buf: &mut [u8]) -> PhonicResult<(usize, usize)> {
        Err(PhonicError::Unsupported)
    }
}

impl<T: Format> FormatWriter for UnWriteable<T> {
    fn write(&mut self, _stream: usize, _buf: &[u8]) -> PhonicResult<usize> {
        Err(PhonicError::Unsupported)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Err(PhonicError::Unsupported)
    }

    fn finalize(&mut self) -> PhonicResult<()> {
        Err(PhonicError::Unsupported)
    }
}

impl<T: Format> FormatSeeker for UnSeekable<T> {
    fn seek(&mut self, _stream: usize, _offset: i64) -> PhonicResult<()> {
        Err(PhonicError::Unsupported)
    }
}

macro_rules! impl_stream {
    ($name:ident) => {
        impl<T: Stream> Stream for $name<T> {
            type Tag = T::Tag;

            fn stream_spec(&self) -> &StreamSpec<Self::Tag> {
                self.0.stream_spec()
            }
        }

        impl<T: IndexedStream> IndexedStream for $name<T> {
            fn pos(&self) -> u64 {
                self.0.pos()
            }
        }
    };
}

macro_rules! impl_finite_stream {
    ($name:ident) => {
        impl<T: FiniteStream> FiniteStream for $name<T> {
            fn len(&self) -> u64 {
                self.0.len()
            }
        }
    };
}

macro_rules! impl_stream_read {
    ($name:ident) => {
        impl<T: StreamReader> StreamReader for $name<T> {
            fn read(&mut self, buf: &mut [u8]) -> PhonicResult<usize> {
                self.0.read(buf)
            }
        }
    };
}

macro_rules! impl_stream_write {
    ($name:ident) => {
        impl<T: StreamWriter> StreamWriter for $name<T> {
            fn write(&mut self, buf: &[u8]) -> PhonicResult<usize> {
                self.0.write(buf)
            }

            fn flush(&mut self) -> PhonicResult<()> {
                self.0.flush()
            }
        }
    };
}

macro_rules! impl_stream_seek {
    ($name:ident) => {
        impl<T: StreamSeeker> StreamSeeker for $name<T> {
            fn seek(&mut self, offset: i64) -> PhonicResult<()> {
                self.0.seek(offset)
            }
        }
    };
}

impl_stream!(Infinite);
impl_stream!(UnReadable);
impl_stream!(UnWriteable);
impl_stream!(UnSeekable);

impl_finite_stream!(UnReadable);
impl_finite_stream!(UnWriteable);
impl_finite_stream!(UnSeekable);

impl_stream_read!(Infinite);
impl_stream_read!(UnWriteable);
impl_stream_read!(UnSeekable);

impl_stream_write!(Infinite);
impl_stream_write!(UnReadable);
impl_stream_write!(UnSeekable);

impl_stream_seek!(Infinite);
impl_stream_seek!(UnReadable);
impl_stream_seek!(UnWriteable);

impl<T: Stream> FiniteStream for Infinite<T> {
    fn len(&self) -> u64 {
        u64::MAX
    }
}

impl<T: Stream> StreamReader for UnReadable<T> {
    fn read(&mut self, _buf: &mut [u8]) -> PhonicResult<usize> {
        Err(PhonicError::Unsupported)
    }
}

impl<T: Stream> StreamWriter for UnWriteable<T> {
    fn write(&mut self, _buf: &[u8]) -> PhonicResult<usize> {
        Err(PhonicError::Unsupported)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Err(PhonicError::Unsupported)
    }
}

impl<T: Stream> StreamSeeker for UnSeekable<T> {
    fn seek(&mut self, _offset: i64) -> PhonicResult<()> {
        Err(PhonicError::Unsupported)
    }
}

macro_rules! impl_signal {
    ($name:ident) => {
        impl<T: Signal> Signal for $name<T> {
            type Sample = T::Sample;

            fn spec(&self) -> &SignalSpec {
                self.0.spec()
            }
        }
    };
}

macro_rules! impl_finite_signal {
    ($name:ident) => {
        impl<T: FiniteSignal> FiniteSignal for $name<T> {
            fn len(&self) -> u64 {
                self.0.len()
            }
        }
    };
}

macro_rules! impl_signal_read {
    ($name:ident) => {
        impl<T: SignalReader> SignalReader for $name<T> {
            fn read(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize> {
                self.0.read(buf)
            }
        }
    };
}

macro_rules! impl_signal_write {
    ($name:ident) => {
        impl<T: SignalWriter> SignalWriter for $name<T> {
            fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
                self.0.write(buf)
            }

            fn flush(&mut self) -> PhonicResult<()> {
                self.0.flush()
            }
        }
    };
}

macro_rules! impl_signal_seek {
    ($name:ident) => {
        impl<T: SignalSeeker> SignalSeeker for $name<T> {
            fn seek(&mut self, offset: i64) -> PhonicResult<()> {
                self.0.seek(offset)
            }
        }
    };
}

impl_signal!(Infinite);
impl_signal!(UnReadable);
impl_signal!(UnWriteable);
impl_signal!(UnSeekable);

impl_finite_signal!(UnReadable);
impl_finite_signal!(UnWriteable);
impl_finite_signal!(UnSeekable);

impl_signal_read!(Infinite);
impl_signal_read!(UnWriteable);
impl_signal_read!(UnSeekable);

impl_signal_write!(Infinite);
impl_signal_write!(UnReadable);
impl_signal_write!(UnSeekable);

impl_signal_seek!(Infinite);
impl_signal_seek!(UnReadable);
impl_signal_seek!(UnWriteable);

impl<T: Signal> FiniteSignal for Infinite<T> {
    fn len(&self) -> u64 {
        u64::MAX
    }
}

impl<T: Signal> SignalReader for UnReadable<T> {
    fn read(&mut self, _buf: &mut [Self::Sample]) -> PhonicResult<usize> {
        Err(PhonicError::Unsupported)
    }
}

impl<T: Signal> SignalWriter for UnWriteable<T> {
    fn write(&mut self, _buf: &[Self::Sample]) -> PhonicResult<usize> {
        Err(PhonicError::Unsupported)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        Err(PhonicError::Unsupported)
    }
}

impl<T: Signal> SignalSeeker for UnSeekable<T> {
    fn seek(&mut self, _offset: i64) -> PhonicResult<()> {
        Err(PhonicError::Unsupported)
    }
}

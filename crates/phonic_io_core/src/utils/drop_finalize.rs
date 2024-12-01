use crate::{
    FiniteFormat, Format, FormatReader, FormatSeeker, FormatTag, FormatWriter, IndexedFormat,
    StreamSpec,
};
use phonic_signal::PhonicResult;

pub struct DropFinalize<T: FormatWriter>(pub T);

impl<T: FormatWriter> Format for DropFinalize<T> {
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

impl<T: FormatWriter + IndexedFormat> IndexedFormat for DropFinalize<T> {
    fn pos(&self) -> u64 {
        self.0.pos()
    }

    fn stream_pos(&self, stream: usize) -> u64 {
        self.0.stream_pos(stream)
    }
}

impl<T: FormatWriter + FiniteFormat> FiniteFormat for DropFinalize<T> {
    fn len(&self) -> u64 {
        self.0.len()
    }

    fn stream_len(&self, stream: usize) -> u64 {
        self.0.stream_len(stream)
    }
}

impl<T: FormatWriter + FormatReader> FormatReader for DropFinalize<T> {
    fn read(&mut self, buf: &mut [u8]) -> PhonicResult<(usize, usize)> {
        self.0.read(buf)
    }
}

impl<T: FormatWriter> FormatWriter for DropFinalize<T> {
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

impl<T: FormatWriter + FormatSeeker> FormatSeeker for DropFinalize<T> {
    fn seek(&mut self, stream: usize, offset: i64) -> PhonicResult<()> {
        self.0.seek(stream, offset)
    }
}

impl<T: FormatWriter> Drop for DropFinalize<T> {
    fn drop(&mut self) {
        let _ = self.finalize();
        let _ = self.flush();
    }
}

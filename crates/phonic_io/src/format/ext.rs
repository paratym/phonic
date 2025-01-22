use crate::{
    block_on_format, BlockingFormat, FiniteFormat, Format, FormatReader, FormatSeeker, FormatTag,
    FormatWriter, IndexedFormat, StreamSpec,
};
use phonic_signal::{utils::slice_as_init_mut, PhonicResult};
use std::mem::MaybeUninit;

pub trait FormatExt: Format {
    fn current_stream_spec(&self) -> &StreamSpec<<Self::Tag as FormatTag>::Codec> {
        let i = self.current_stream();
        &self.streams()[i]
    }

    fn primary_stream_spec(&self) -> Option<&StreamSpec<<Self::Tag as FormatTag>::Codec>> {
        self.primary_stream().and_then(|i| self.streams().get(i))
    }

    fn is_empty(&self) -> bool
    where
        Self: FiniteFormat,
    {
        self.len() == 0
    }

    fn stream_is_empty(&self, stream: usize) -> bool
    where
        Self: FiniteFormat,
    {
        self.stream_len(stream) == 0
    }

    fn is_exhausted(&self) -> bool
    where
        Self: IndexedFormat + FiniteFormat,
    {
        self.pos() == self.len()
    }

    fn stream_is_exhausted(&self, stream: usize) -> bool
    where
        Self: IndexedFormat + FiniteFormat,
    {
        self.stream_pos(stream) == self.stream_len(stream)
    }

    fn read_init<'a>(
        &mut self,
        buf: &'a mut [MaybeUninit<u8>],
    ) -> PhonicResult<(usize, &'a mut [u8])>
    where
        Self: FormatReader,
    {
        let (stream_i, n_bytes) = self.read(buf)?;
        let uninit_slice = &mut buf[..n_bytes];
        let init_slice = unsafe { slice_as_init_mut(uninit_slice) };

        Ok((stream_i, init_slice))
    }

    fn read_blocking(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<(usize, usize)>
    where
        Self: BlockingFormat + FormatReader,
    {
        block_on_format!(self, self.read(buf))
    }

    fn write_blocking(&mut self, stream: usize, buf: &[u8]) -> PhonicResult<usize>
    where
        Self: BlockingFormat + FormatWriter,
    {
        block_on_format!(self, self.write(stream, buf))
    }

    fn flush_blocking(&mut self) -> PhonicResult<()>
    where
        Self: BlockingFormat + FormatWriter,
    {
        block_on_format!(self, self.flush())
    }

    fn finalize_blocking(&mut self) -> PhonicResult<()>
    where
        Self: BlockingFormat + FormatWriter,
    {
        block_on_format!(self, self.finalize())
    }

    fn set_pos(&mut self, stream: usize, pos: u64) -> Result<(), phonic_signal::PhonicError>
    where
        Self: Sized + IndexedFormat + FormatSeeker,
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

impl<T: Format> FormatExt for T {}

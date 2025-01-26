use crate::{
    formats::wave::{
        update_nested_chunk_header, FmtChunk, RiffChunk, WaveFormatTag, WaveSupportedCodec,
    },
    FiniteFormat, FiniteStream, Format, FormatFromReader, FormatFromWriter, FormatReader,
    FormatSeeker, FormatTag, FormatWriter, IndexedFormat, IndexedStream, Stream, StreamReader,
    StreamSeeker, StreamSpec, StreamSpecBuilder, StreamWriter,
};
use phonic_signal::{utils::slice_as_init_mut, PhonicError, PhonicResult};
use std::{
    io::{Read, Seek, Write},
    mem::MaybeUninit,
};

pub struct WaveFormat<T, F: FormatTag = WaveFormatTag> {
    tag: F,
    spec: StreamSpec<F::Codec>,
    data: RiffChunk<RiffChunk<T>>,
}

impl<T, F: FormatTag> WaveFormat<T, F> {
    const RIFF_CHUNK_ID: [u8; 4] = *b"RIFF";
    const WAVE_ID: [u8; 4] = *b"WAVE";
    const DATA_CHUNK_ID: [u8; 4] = *b"data";

    pub fn into_inner(self) -> T {
        self.data.into_inner().into_inner()
    }

    fn read_header(
        reader: T,
        spec: &mut StreamSpecBuilder<F::Codec>,
    ) -> PhonicResult<RiffChunk<RiffChunk<T>>>
    where
        T: Read,
        WaveSupportedCodec: TryInto<F::Codec>,
        PhonicError: From<<WaveSupportedCodec as TryInto<F::Codec>>::Error>,
    {
        let mut riff_chunk = RiffChunk::read_new(reader)?;
        if riff_chunk.id() != Self::RIFF_CHUNK_ID {
            return Err(PhonicError::invalid_data());
        }

        let mut wave_id = [0u8; 4];
        riff_chunk.read_exact(&mut wave_id)?;
        if wave_id != Self::WAVE_ID {
            return Err(PhonicError::invalid_data());
        }

        loop {
            let mut chunk = RiffChunk::read_new(riff_chunk)?;
            match chunk.id() {
                FmtChunk::CHUNK_ID => FmtChunk::read_riff_chunk(&mut chunk)
                    .map_err(Into::into)
                    .and_then(|fmt| fmt.apply_to_spec(spec))?,

                Self::DATA_CHUNK_ID => break Ok(chunk),
                _ => chunk.skip_remaining()?,
            };

            riff_chunk = chunk.into_inner();
        }
    }

    fn write_header(writer: T, spec: StreamSpec<F::Codec>) -> PhonicResult<RiffChunk<RiffChunk<T>>>
    where
        T: Write + Seek,
        F::Codec: TryInto<WaveSupportedCodec>,
        PhonicError: From<<F::Codec as TryInto<WaveSupportedCodec>>::Error>,
    {
        let mut riff_chunk = RiffChunk::write_new(writer, Self::RIFF_CHUNK_ID)?;
        riff_chunk.write_all(&Self::WAVE_ID)?;

        let fmt = FmtChunk::try_from_spec(spec)?;
        fmt.write_riff_chunk(&mut riff_chunk)?;

        RiffChunk::write_new(riff_chunk, Self::DATA_CHUNK_ID).map_err(Into::into)
    }
}

impl<T, F> FormatFromReader<T, F> for WaveFormat<T, F>
where
    T: Read,
    F: FormatTag,
    WaveFormatTag: TryInto<F>,
    WaveSupportedCodec: TryInto<F::Codec>,
    PhonicError: From<<WaveFormatTag as TryInto<F>>::Error>,
    PhonicError: From<<WaveSupportedCodec as TryInto<F::Codec>>::Error>,
{
    fn read_index(reader: T) -> PhonicResult<Self> {
        let tag = WaveFormatTag.try_into()?;

        let mut spec_builder = StreamSpec::builder();
        let data = Self::read_header(reader, &mut spec_builder)?;
        let spec = spec_builder.build()?;

        Ok(Self { tag, spec, data })
    }
}

impl<T, F> FormatFromWriter<T, F> for WaveFormat<T, F>
where
    T: Write + Seek,
    F: FormatTag,
    WaveFormatTag: TryInto<F>,
    F::Codec: TryInto<WaveSupportedCodec>,
    PhonicError: From<<WaveFormatTag as TryInto<F>>::Error>,
    PhonicError: From<<F::Codec as TryInto<WaveSupportedCodec>>::Error>,
{
    fn write_index<I>(writer: T, index: I) -> PhonicResult<Self>
    where
        I: IntoIterator<Item = StreamSpec<F::Codec>>,
    {
        let tag = WaveFormatTag.try_into()?;

        let mut index_iter = index.into_iter();
        let spec = index_iter.next().ok_or(PhonicError::missing_data())?;
        if index_iter.next().is_some() {
            return Err(PhonicError::unsupported());
        }

        let data = Self::write_header(writer, spec)?;

        Ok(Self { tag, spec, data })
    }
}

impl<T, F: FormatTag> Format for WaveFormat<T, F> {
    type Tag = F;

    fn format(&self) -> Self::Tag {
        self.tag
    }

    fn streams(&self) -> &[StreamSpec<<Self::Tag as FormatTag>::Codec>] {
        std::slice::from_ref(&self.spec)
    }

    fn current_stream(&self) -> usize {
        0
    }

    fn primary_stream(&self) -> Option<usize> {
        Some(0)
    }
}

impl<T, F> IndexedFormat for WaveFormat<T, F>
where
    F: FormatTag,
    Self: Format<Tag = F> + IndexedStream<Tag = F::Codec>,
{
    fn pos(&self) -> u64 {
        IndexedStream::pos(self)
    }

    fn stream_pos(&self, stream: usize) -> u64 {
        match stream {
            0 => IndexedStream::pos(self),
            _ => 0,
        }
    }
}

impl<T, F> FiniteFormat for WaveFormat<T, F>
where
    F: FormatTag,
    Self: Format<Tag = F> + FiniteStream<Tag = F::Codec>,
{
    fn len(&self) -> u64 {
        FiniteStream::len(self)
    }

    fn stream_len(&self, stream: usize) -> u64 {
        match stream {
            0 => FiniteStream::len(self),
            _ => 0,
        }
    }
}

impl<T, F> FormatReader for WaveFormat<T, F>
where
    T: Read,
    F: FormatTag,
    Self: Format<Tag = F> + StreamReader<Tag = F::Codec>,
{
    fn read(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<(usize, usize)> {
        let n = StreamReader::read(self, buf)?;
        Ok((0, n))
    }
}

impl<T, F> FormatWriter for WaveFormat<T, F>
where
    T: Write + Seek,
    F: FormatTag,
    Self: Format<Tag = F> + StreamWriter<Tag = F::Codec>,
{
    fn write(&mut self, stream: usize, buf: &[u8]) -> PhonicResult<usize> {
        match stream {
            0 => StreamWriter::write(self, buf),
            _ => Err(PhonicError::invalid_input()),
        }
    }

    fn flush(&mut self) -> PhonicResult<()> {
        StreamWriter::flush(self)
    }

    fn finalize(&mut self) -> PhonicResult<()> {
        update_nested_chunk_header(&mut self.data).map_err(Into::into)
    }
}

impl<T, F> FormatSeeker for WaveFormat<T, F>
where
    T: Seek,
    F: FormatTag,
    Self: Format<Tag = F> + StreamSeeker<Tag = F::Codec>,
{
    fn seek(&mut self, stream: usize, offset: i64) -> PhonicResult<()> {
        match stream {
            0 => StreamSeeker::seek(self, offset),
            _ => Err(PhonicError::invalid_input()),
        }
    }
}

impl<T, F: FormatTag> Stream for WaveFormat<T, F> {
    type Tag = F::Codec;

    fn stream_spec(&self) -> &StreamSpec<Self::Tag> {
        &self.spec
    }
}

impl<T, F: FormatTag> IndexedStream for WaveFormat<T, F> {
    fn pos(&self) -> u64 {
        self.data.pos() as u64
    }
}

impl<T, F: FormatTag> FiniteStream for WaveFormat<T, F> {
    fn len(&self) -> u64 {
        self.data.len() as u64
    }
}

impl<T: Read, F: FormatTag> StreamReader for WaveFormat<T, F> {
    fn read(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize> {
        let mut len = buf.len();
        len -= len % self.stream_spec().block_align;

        // TODO: this is probably bad. run in miri eventually...
        let uninit_buf = &mut buf[..len];
        let init_buf = unsafe { slice_as_init_mut(uninit_buf) };

        let mut n_bytes = 0;
        loop {
            match self.data.read(&mut init_buf[n_bytes..])? {
                0 if n_bytes == 0 => break,
                0 => return Err(PhonicError::invalid_state()),
                n_read => n_bytes += n_read,
            }

            if n_bytes % self.spec.block_align == 0 {
                break;
            }
        }

        Ok(n_bytes)
    }
}

impl<T: Write + Seek, F: FormatTag> StreamWriter for WaveFormat<T, F> {
    fn write(&mut self, buf: &[u8]) -> PhonicResult<usize> {
        let mut len = buf.len();
        len -= len % self.stream_spec().block_align;

        let mut n_bytes = 0;
        loop {
            match self.data.write(&buf[n_bytes..len])? {
                0 if n_bytes == 0 => break,
                0 => return Err(PhonicError::invalid_state()),
                n_written => n_bytes += n_written,
            }

            if n_bytes % self.spec.block_align == 0 {
                break;
            }
        }

        Ok(n_bytes)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.data.flush().map_err(Into::into)
    }
}

impl<T: Seek, F: FormatTag> StreamSeeker for WaveFormat<T, F> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        if offset % self.spec.block_align as i64 != 0 {
            return Err(PhonicError::invalid_input());
        }

        self.data.seek_relative(offset).map_err(Into::into)
    }
}

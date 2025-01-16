use crate::{
    codec::pcm::PcmCodecTag, CodecTag, Decoder, Encoder, FiniteStream, IndexedStream, Stream,
    StreamReader, StreamSeeker, StreamSpec, StreamSpecBuilder, StreamWriter,
};
use phonic_signal::{
    FiniteSignal, IndexedSignal, PhonicError, PhonicResult, Sample, Signal, SignalReader,
    SignalSeeker, SignalSpec, SignalWriter,
};
use std::{
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
};

pub struct PcmCodec<T, S: Sample, C: CodecTag = PcmCodecTag> {
    inner: T,
    spec: StreamSpec<C>,
    _sample: PhantomData<S>,
}

impl<T, S: Sample, C: CodecTag> Encoder<T, C> for PcmCodec<T, S, C>
where
    PcmCodecTag: TryInto<C>,
    PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
{
    fn encoder(inner: T) -> PhonicResult<Self>
    where
        T: Signal,
    {
        let spec_builder = StreamSpecBuilder::from(&inner);
        let spec = PcmCodecTag::infer_tagged_spec(spec_builder)?;

        Ok(Self {
            inner,
            spec,
            _sample: PhantomData,
        })
    }
}

impl<T, S: Sample, C: CodecTag> Decoder<T, C> for PcmCodec<T, S, C>
where
    PcmCodecTag: TryInto<C>,
    PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
{
    fn decoder(inner: T) -> PhonicResult<Self>
    where
        T: Stream<Tag = C>,
    {
        let spec_builder = inner.stream_spec().into_builder();
        let spec = PcmCodecTag::infer_tagged_spec(spec_builder)?;

        Ok(Self {
            inner,
            spec,
            _sample: PhantomData,
        })
    }
}

impl<T, S: Sample, C: CodecTag> PcmCodec<T, S, C> {
    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T, S: Sample, C: CodecTag> Signal for PcmCodec<T, S, C> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec.decoded_spec
    }
}

impl<T: IndexedStream, S: Sample, C: CodecTag> IndexedSignal for PcmCodec<T, S, C> {
    fn pos(&self) -> u64 {
        self.inner.pos() / size_of::<S>() as u64
    }
}

impl<T: FiniteStream, S: Sample, C: CodecTag> FiniteSignal for PcmCodec<T, S, C> {
    fn len(&self) -> u64 {
        self.inner.len() / size_of::<S>() as u64
    }
}

impl<T, S, C> SignalReader for PcmCodec<T, S, C>
where
    T: StreamReader,
    S: Sample,
    C: CodecTag,
{
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let (leading, aligned, _) = unsafe { buf.align_to_mut::<MaybeUninit<u8>>() };
        assert!(leading.is_empty());

        let n_bytes = self.inner.read(aligned)?;
        assert_eq!(n_bytes % self.stream_spec().block_align, 0);
        assert_eq!(n_bytes % size_of::<S>(), 0);

        Ok(n_bytes / size_of::<S>())
    }
}

impl<T, S, C> SignalWriter for PcmCodec<T, S, C>
where
    T: StreamWriter,
    S: Sample,
    C: CodecTag,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let (leading, aligned, _) = unsafe { buf.align_to::<u8>() };
        assert!(leading.is_empty());

        let n_bytes = self.inner.write(aligned)?;
        debug_assert_eq!(n_bytes % self.stream_spec().block_align, 0);

        Ok(n_bytes / size_of::<S>())
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush().map_err(Into::into)
    }
}

impl<T: StreamSeeker, S: Sample, C: CodecTag> SignalSeeker for PcmCodec<T, S, C> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        self.inner.seek(offset * size_of::<S>() as i64)
    }
}

impl<T, S: Sample, C: CodecTag> Stream for PcmCodec<T, S, C> {
    type Tag = C;

    fn stream_spec(&self) -> &StreamSpec<Self::Tag> {
        &self.spec
    }
}

impl<T, S, C> IndexedStream for PcmCodec<T, S, C>
where
    T: IndexedSignal<Sample = S>,
    S: Sample,
    C: CodecTag,
{
    fn pos(&self) -> u64 {
        self.inner.pos() * size_of::<S>() as u64
    }
}

impl<T, S, C> FiniteStream for PcmCodec<T, S, C>
where
    T: FiniteSignal<Sample = S>,
    S: Sample,
    C: CodecTag,
{
    fn len(&self) -> u64 {
        self.inner.len() * size_of::<S>() as u64
    }
}

impl<T, S, C> StreamReader for PcmCodec<T, S, C>
where
    T: SignalReader<Sample = S>,
    S: Sample,
    C: CodecTag,
{
    fn read(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize> {
        let (leading, aligned, _) = unsafe { buf.align_to_mut::<MaybeUninit<S>>() };
        let mut n_samples = 0;

        loop {
            match self.inner.read(aligned)? {
                0 if n_samples == 0 => break,
                0 => return Err(PhonicError::InvalidData),
                n => n_samples += n,
            }

            if n_samples % self.spec.block_align == 0 {
                break;
            }
        }

        let offset = leading.len();
        buf.rotate_left(offset);

        Ok(n_samples * size_of::<S>())
    }
}

impl<T, S, C> StreamWriter for PcmCodec<T, S, C>
where
    T: SignalWriter<Sample = S>,
    S: Sample,
    C: CodecTag,
{
    fn write(&mut self, buf: &[u8]) -> PhonicResult<usize> {
        //         let start_i = size_of::<S>() - (buf.as_ptr() as usize % align_of::<S>());
        //         let aligned_len = buf.len() - start_i;
        //         let usable_len = aligned_len - (aligned_len % size_of::<S>());

        //         let sample_buf = match buf[start_i..start_i + usable_len].as_slice_of::<S>() {
        //             Ok(buf) => buf,
        //             _ => return Err(io::ErrorKind::InvalidData.into()),
        //         };

        //         let n = self.inner.write(sample_buf)?;
        //         Ok(n * size_of::<S>())
        todo!()
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush()
    }
}

impl<T, S, C> StreamSeeker for PcmCodec<T, S, C>
where
    T: SignalSeeker<Sample = S>,
    S: Sample,
    C: CodecTag,
{
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        self.inner.seek(offset / size_of::<S>() as i64)
    }
}

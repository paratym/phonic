use crate::{
    codecs::pcm::{ArbitrarySample, Endianess, PcmCodecTag},
    CodecFromSignal, CodecFromStream, CodecTag, FiniteStream, IndexedStream, Stream, StreamReader,
    StreamSeeker, StreamSpec, StreamSpecBuilder, StreamWriter,
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
    endianess: Endianess,
    _sample: PhantomData<S>,
}

impl<T, S: Sample, C: CodecTag> PcmCodec<T, S, C> {
    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T, S, C> CodecFromSignal<T, C> for PcmCodec<T, S, C>
where
    T: Signal,
    S: Sample,
    C: CodecTag + TryInto<PcmCodecTag>,
    PcmCodecTag: TryInto<C>,
    PhonicError: From<<C as TryInto<PcmCodecTag>>::Error>,
    PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
{
    fn from_signal(tag: C, inner: T) -> PhonicResult<Self>
    where
        T: Signal,
    {
        let spec_builder = StreamSpecBuilder::from(&inner).with_codec(tag);
        let spec = PcmCodecTag::infer_tagged_spec(spec_builder)?;

        let native_tag: PcmCodecTag = spec.codec.try_into()?;
        let endianess = Endianess::from(native_tag);

        Ok(Self {
            inner,
            spec,
            endianess,
            _sample: PhantomData,
        })
    }
}

impl<T, S, C> CodecFromStream<T, C> for PcmCodec<T, S, C>
where
    T: Stream<Tag = C>,
    S: Sample,
    C: CodecTag + TryInto<PcmCodecTag>,
    PcmCodecTag: TryInto<C>,
    PhonicError: From<<C as TryInto<PcmCodecTag>>::Error>,
    PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
{
    fn from_stream(inner: T) -> PhonicResult<Self> {
        let spec_builder = inner.stream_spec().into_builder();
        let spec = PcmCodecTag::infer_tagged_spec(spec_builder)?;

        let native_tag: PcmCodecTag = spec.codec.try_into()?;
        let endianess = Endianess::from(native_tag);

        Ok(Self {
            inner,
            spec,
            endianess,
            _sample: PhantomData,
        })
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
    S: Sample + ArbitrarySample,
    C: CodecTag,
{
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let (prefix, aligned, suffix) = unsafe { buf.align_to_mut::<MaybeUninit<u8>>() };
        debug_assert!(prefix.is_empty() && suffix.is_empty());

        let aligned_len = aligned.len() - aligned.len() % self.spec.block_align;
        if aligned_len == 0 {
            return Err(PhonicError::invalid_input());
        }

        let mut n_bytes = 0;
        loop {
            match self.inner.read(&mut aligned[n_bytes..aligned_len]) {
                Ok(0) if n_bytes == 0 => break,
                Ok(0) => return Err(PhonicError::invalid_state()),
                Ok(n) => n_bytes += n,
                Err(e) => return Err(e),
            };

            if n_bytes % self.spec.block_align == 0 {
                break;
            }
        }

        if !self.endianess.is_native() {
            for i in (0..n_bytes).step_by(size_of::<S>()) {
                buf[i..i + size_of::<S>()].reverse()
            }
        }

        debug_assert_eq!(n_bytes % size_of::<S>(), 0);
        Ok(n_bytes / size_of::<S>())
    }
}

impl<T, S, C> SignalWriter for PcmCodec<T, S, C>
where
    T: StreamWriter,
    S: Sample + ArbitrarySample,
    C: CodecTag,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        if !self.endianess.is_native() {
            return Err(PhonicError::unsupported());
        }

        let (prefix, aligned, suffix) = unsafe { buf.align_to::<u8>() };
        debug_assert!(prefix.is_empty() && suffix.is_empty());

        let aligned_len = aligned.len() - aligned.len() % self.spec.block_align;
        if aligned_len == 0 {
            return Err(PhonicError::invalid_input());
        }

        let mut n_bytes = 0;
        loop {
            match self.inner.write(&aligned[n_bytes..aligned_len]) {
                Ok(0) if n_bytes == 0 => break,
                Ok(0) => return Err(PhonicError::invalid_state()),
                Ok(n) => n_bytes += n,
                Err(e) => return Err(e),
            }

            if n_bytes % self.spec.block_align == 0 {
                break;
            }
        }

        debug_assert_eq!(n_bytes % size_of::<S>(), 0);
        Ok(n_bytes / size_of::<S>())
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush()
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
    S: Sample + ArbitrarySample,
    C: CodecTag,
{
    fn read(&mut self, buf: &mut [MaybeUninit<u8>]) -> PhonicResult<usize> {
        let (leading, aligned, _) = unsafe { buf.align_to_mut::<MaybeUninit<S>>() };

        let aligned_byte_len = aligned.len() - aligned.len() % self.spec.block_align;
        debug_assert_eq!(aligned_byte_len % size_of::<S>(), 0);
        let aligned_len = aligned_byte_len / size_of::<S>();
        if aligned_len == 0 {
            return Err(PhonicError::invalid_input());
        }

        let mut n_samples = 0;
        loop {
            match self.inner.read(&mut aligned[n_samples..aligned_len]) {
                Ok(0) if n_samples == 0 => break,
                Ok(0) => return Err(PhonicError::invalid_state()),
                Ok(n) => n_samples += n,
                Err(e) => return Err(e),
            }

            if n_samples % self.spec.block_align == 0 {
                break;
            }
        }

        let n_bytes = n_samples * size_of::<S>();
        let offset = leading.len();
        buf.rotate_left(offset);

        if !self.endianess.is_native() {
            for i in (0..n_bytes).step_by(size_of::<S>()) {
                buf[i..i + size_of::<S>()].reverse()
            }
        }

        Ok(n_bytes)
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

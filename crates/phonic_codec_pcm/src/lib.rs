use byte_slice_cast::{
    AsByteSlice, AsMutByteSlice, AsMutSliceOf, FromByteSlice, ToByteSlice, ToMutByteSlice,
};
use phonic_core::PhonicError;
use phonic_io_core::{
    match_tagged_signal, CodecTag, DynStream, KnownSampleType, Stream, StreamObserver,
    StreamReader, StreamSeeker, StreamSpec, StreamWriter, TaggedSignal,
};
use phonic_signal::{
    Sample, Signal, SignalObserver, SignalReader, SignalSeeker, SignalSpec, SignalWriter,
};
use std::{
    marker::PhantomData,
    mem::{align_of, size_of},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct PcmCodecTag;

pub struct PcmCodec<T, S: Sample, C: CodecTag = PcmCodecTag> {
    inner: T,
    stream_spec: StreamSpec<C>,
    signal_spec: SignalSpec,
    _sample: PhantomData<S>,
}

pub fn fill_pcm_spec<C>(spec: &mut StreamSpec<C>) -> Result<(), PhonicError>
where
    C: CodecTag,
    PcmCodecTag: TryInto<C>,
{
    let expected_codec = PcmCodecTag.try_into().ok();
    if spec.codec.is_some() && spec.codec != expected_codec {
        return Err(PhonicError::InvalidData);
    } else {
        spec.codec = expected_codec;
    }

    let sample_type = spec
        .sample_type
        .and_then(|s| KnownSampleType::try_from(s).ok());

    if sample_type.is_none() {
        return Ok(());
    }

    let sample_byte_size = sample_type.unwrap().byte_size();
    let calculated_bitrate = spec
        .decoded_spec
        .raw_sample_rate()
        .map(|r| sample_byte_size as f64 * 8.0 * r as f64);

    if calculated_bitrate.is_some_and(|rate| spec.avg_bitrate.get_or_insert(rate) != &rate) {
        return Err(PhonicError::InvalidData);
    }

    let calculated_block_align = spec
        .decoded_spec
        .channels
        .map(|c| c.count() as u16 * sample_byte_size as u16);

    if calculated_block_align
        .is_some_and(|align| *spec.block_align.get_or_insert(align) % align != 0)
    {
        return Err(PhonicError::InvalidData);
    }

    Ok(())
}

pub fn pcm_codec_from_stream<S>(stream: S) -> Result<TaggedSignal, PhonicError>
where
    S: DynStream + 'static,
    PcmCodecTag: TryInto<S::Tag>,
{
    let signal = match stream
        .spec()
        .sample_type
        .ok_or(PhonicError::MissingData)?
        .try_into()?
    {
        KnownSampleType::I8 => TaggedSignal::I8(Box::new(PcmCodec::from_stream(stream)?)),
        KnownSampleType::I16 => TaggedSignal::I16(Box::new(PcmCodec::from_stream(stream)?)),
        KnownSampleType::I32 => TaggedSignal::I32(Box::new(PcmCodec::from_stream(stream)?)),
        KnownSampleType::I64 => TaggedSignal::I64(Box::new(PcmCodec::from_stream(stream)?)),
        KnownSampleType::U8 => TaggedSignal::U8(Box::new(PcmCodec::from_stream(stream)?)),
        KnownSampleType::U16 => TaggedSignal::U16(Box::new(PcmCodec::from_stream(stream)?)),
        KnownSampleType::U32 => TaggedSignal::U32(Box::new(PcmCodec::from_stream(stream)?)),
        KnownSampleType::U64 => TaggedSignal::U64(Box::new(PcmCodec::from_stream(stream)?)),
        KnownSampleType::F32 => TaggedSignal::F32(Box::new(PcmCodec::from_stream(stream)?)),
        KnownSampleType::F64 => TaggedSignal::F64(Box::new(PcmCodec::from_stream(stream)?)),
    };

    Ok(signal)
}

pub fn pcm_codec_from_signal<C>(
    signal: TaggedSignal,
) -> Result<Box<dyn DynStream<Tag = C>>, PhonicError>
where
    C: CodecTag + 'static,
    PcmCodecTag: TryInto<C>,
{
    match_tagged_signal!(signal, inner => Ok(Box::new(PcmCodec::from_signal(inner)?)))
}

impl CodecTag for PcmCodecTag {
    fn fill_spec(spec: &mut StreamSpec<Self>) -> Result<(), PhonicError> {
        fill_pcm_spec(spec)
    }
}

impl<T, S: Sample, C: CodecTag> PcmCodec<T, S, C> {
    pub fn from_stream(inner: T) -> Result<Self, PhonicError>
    where
        T: Stream<Tag = C>,
        PcmCodecTag: TryInto<C>,
    {
        let mut stream_spec = *inner.spec();
        fill_pcm_spec(&mut stream_spec)?;
        let signal_spec = stream_spec.decoded_spec.build()?;

        Ok(Self {
            inner,
            stream_spec,
            signal_spec,
            _sample: PhantomData,
        })
    }

    pub fn from_signal(inner: T) -> Result<Self, PhonicError>
    where
        T: Signal<Sample = S>,
        T::Sample: 'static,
        PcmCodecTag: TryInto<C>,
    {
        let signal_spec = *inner.spec();
        let mut stream_spec = StreamSpec::<C>::from(&inner);
        fill_pcm_spec(&mut stream_spec)?;

        Ok(Self {
            inner,
            stream_spec,
            signal_spec,
            _sample: PhantomData,
        })
    }

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
        &self.signal_spec
    }
}

impl<T: StreamObserver, S: Sample, C: CodecTag> SignalObserver for PcmCodec<T, S, C> {
    fn position(&self) -> Result<u64, PhonicError> {
        Ok(self.inner.position()? / size_of::<S>() as u64)
    }
}

impl<T, S, C> SignalReader for PcmCodec<T, S, C>
where
    T: StreamReader,
    S: Sample + ToMutByteSlice,
    C: CodecTag,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let byte_buf = buf.as_mut_byte_slice();
        let n = self.inner.read(byte_buf)?;

        let bytes_per_sample = byte_buf.len() / buf.len();
        if n % bytes_per_sample != 0 {
            todo!()
        }

        Ok(n / bytes_per_sample)
    }
}

impl<T, S, C> SignalWriter for PcmCodec<T, S, C>
where
    T: StreamWriter,
    S: Sample + ToByteSlice,
    C: CodecTag,
{
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, PhonicError> {
        let byte_buf = buf.as_byte_slice();
        let n = self.inner.write(byte_buf)?;

        let bytes_per_sample = byte_buf.len() / buf.len();
        if n % bytes_per_sample != 0 {
            todo!()
        }

        Ok(n / bytes_per_sample)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.inner.flush().map_err(Into::into)
    }
}

impl<T: StreamSeeker, S: Sample, C: CodecTag> SignalSeeker for PcmCodec<T, S, C> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        self.inner.seek(offset * size_of::<S>() as i64)
    }
}

impl<T, S: Sample, C: CodecTag> Stream for PcmCodec<T, S, C> {
    type Tag = C;

    fn spec(&self) -> &StreamSpec<Self::Tag> {
        &self.stream_spec
    }
}

impl<T: SignalObserver, S: Sample, C: CodecTag> StreamObserver for PcmCodec<T, S, C> {
    fn position(&self) -> Result<u64, PhonicError> {
        Ok(self.inner.position()? / size_of::<S>() as u64)
    }
}

impl<T, S, C> StreamReader for PcmCodec<T, S, C>
where
    T: SignalReader<Sample = S>,
    S: Sample + FromByteSlice,
    C: CodecTag,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, PhonicError> {
        let start_i = size_of::<S>() - (buf.as_ptr() as usize % align_of::<S>());
        let aligned_len = buf.len() - start_i;
        let usable_len = aligned_len - (aligned_len % size_of::<S>());

        let sample_buf = match buf[start_i..start_i + usable_len].as_mut_slice_of::<S>() {
            Ok(buf) => buf,
            _ => return Err(PhonicError::InvalidData),
        };

        let n = self.inner.read(sample_buf)?;
        if start_i > 0 {
            buf.rotate_left(start_i);
        }

        Ok(n * size_of::<S>())
    }
}

impl<T, S, C> StreamWriter for PcmCodec<T, S, C>
where
    T: SignalWriter<Sample = S>,
    S: Sample + FromByteSlice,
    C: CodecTag,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize, PhonicError> {
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

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.inner.flush()
    }
}

impl<T: SignalSeeker, S: Sample, C: CodecTag> StreamSeeker for PcmCodec<T, S, C> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        self.inner.seek(offset / size_of::<S>() as i64)
    }
}

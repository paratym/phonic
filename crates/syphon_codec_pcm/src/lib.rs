use byte_slice_cast::{
    AsByteSlice, AsMutByteSlice, AsMutSliceOf, FromByteSlice, ToByteSlice, ToMutByteSlice,
};
use std::{
    marker::PhantomData,
    mem::{align_of, size_of},
};
use syphon_core::SyphonError;
use syphon_io_core::{
    CodecTag, Stream, StreamObserver, StreamReader, StreamSeeker, StreamSpec, StreamWriter,
};
use syphon_signal::{
    KnownSampleType, Sample, Signal, SignalObserver, SignalReader, SignalSeeker, SignalSpec,
    SignalWriter,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PcmCodecTag();
impl CodecTag for PcmCodecTag {}

pub fn fill_pcm_stream_spec<C>(spec: &mut StreamSpec<C>) -> Result<(), SyphonError>
where
    C: CodecTag,
    PcmCodecTag: Into<C>,
{
    let expected_codec = PcmCodecTag().into();
    if spec.codec.get_or_insert(expected_codec) != &expected_codec {
        return Err(SyphonError::InvalidData);
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
        .sample_rate()
        .map(|r| sample_byte_size as f64 * 8.0 * r as f64);

    if calculated_bitrate.is_some_and(|rate| spec.avg_bitrate.get_or_insert(rate) != &rate) {
        return Err(SyphonError::InvalidData);
    }

    let calculated_block_align = spec
        .decoded_spec
        .channels
        .map(|c| c.count() as u16 * sample_byte_size as u16);

    if calculated_block_align
        .is_some_and(|align| *spec.block_align.get_or_insert(align) % align != 0)
    {
        return Err(SyphonError::InvalidData);
    }

    Ok(())
}

pub struct PcmCodec<T, S: Sample, C: CodecTag = PcmCodecTag> {
    inner: T,
    stream_spec: StreamSpec<C>,
    signal_spec: SignalSpec,
    _sample: PhantomData<S>,
}

impl<T, S: Sample, C: CodecTag> PcmCodec<T, S, C> {
    pub fn from_stream(inner: T) -> Result<Self, SyphonError>
    where
        T: Stream<Tag = C>,
        PcmCodecTag: Into<C>,
    {
        let mut stream_spec = *inner.spec();
        fill_pcm_stream_spec(&mut stream_spec)?;
        let signal_spec = stream_spec.decoded_spec.build()?;

        Ok(Self {
            inner,
            stream_spec,
            signal_spec,
            _sample: PhantomData,
        })
    }

    pub fn from_signal(inner: T) -> Result<Self, SyphonError>
    where
        T: Signal,
        T::Sample: 'static,
        PcmCodecTag: Into<C>,
    {
        let signal_spec = *inner.spec();
        let mut stream_spec = StreamSpec::<C>::from(&inner);
        fill_pcm_stream_spec(&mut stream_spec)?;

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

impl<T: SignalObserver, S: Sample, C: CodecTag> SignalObserver for PcmCodec<T, S, C> {
    fn position(&self) -> Result<u64, SyphonError> {
        Ok(self.inner.position()? / size_of::<S>() as u64)
    }
}

impl<T, S, C> SignalReader for PcmCodec<T, S, C>
where
    T: StreamReader,
    S: Sample + ToMutByteSlice,
    C: CodecTag,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, SyphonError> {
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
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, SyphonError> {
        let byte_buf = buf.as_byte_slice();
        let n = self.inner.write(byte_buf)?;

        let bytes_per_sample = byte_buf.len() / buf.len();
        if n % bytes_per_sample != 0 {
            todo!()
        }

        Ok(n / bytes_per_sample)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.inner.flush().map_err(Into::into)
    }
}

impl<T: StreamSeeker, S: Sample, C: CodecTag> SignalSeeker for PcmCodec<T, S, C> {
    fn seek(&mut self, offset: i64) -> Result<(), SyphonError> {
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
    fn position(&self) -> Result<u64, SyphonError> {
        Ok(self.inner.position()? / size_of::<S>() as u64)
    }
}

impl<T, S, C> StreamReader for PcmCodec<T, S, C>
where
    T: SignalReader<Sample = S>,
    S: Sample + FromByteSlice,
    C: CodecTag,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SyphonError> {
        let start_i = size_of::<S>() - (buf.as_ptr() as usize % align_of::<S>());
        let aligned_len = buf.len() - start_i;
        let usable_len = aligned_len - (aligned_len % size_of::<S>());

        let sample_buf = match buf[start_i..start_i + usable_len].as_mut_slice_of::<S>() {
            Ok(buf) => buf,
            _ => return Err(SyphonError::InvalidData),
        };

        let n = self.inner.read(sample_buf)?;
        buf.rotate_left(start_i);
        Ok(n * size_of::<S>())
    }
}

// impl<T, S> Write for PcmCodec<T, S>
// where
//     T: SignalWriter<Sample = S>,
//     S: Sample + FromByteSlice,
// {
//     fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
//         let start_i = size_of::<S>() - (buf.as_ptr() as usize % align_of::<S>());
//         let aligned_len = buf.len() - start_i;
//         let usable_len = aligned_len - (aligned_len % size_of::<S>());

//         let sample_buf = match buf[start_i..start_i + usable_len].as_slice_of::<S>() {
//             Ok(buf) => buf,
//             _ => return Err(io::ErrorKind::InvalidData.into()),
//         };

//         let n = self.inner.write(sample_buf)?;
//         Ok(n * size_of::<S>())
//     }

//     fn flush(&mut self) -> io::Result<()> {
//         self.inner.flush().map_err(Into::into)
//     }
// }

impl<T: SignalSeeker, S: Sample, C: CodecTag> StreamSeeker for PcmCodec<T, S, C> {
    fn seek(&mut self, offset: i64) -> Result<(), SyphonError> {
        self.inner.seek(offset / size_of::<S>() as i64)
    }
}

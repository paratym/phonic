use crate::{
    io::{
        codecs::{CodecTag, SyphonCodec},
        KnownSampleType, Stream, StreamSpec,
    },
    signal::{Sample, Signal, SignalReader, SignalSpec, SignalWriter},
    SyphonError,
};
use byte_slice_cast::{
    AsByteSlice, AsMutByteSlice, AsMutSliceOf, FromByteSlice, ToByteSlice, ToMutByteSlice,
};
use std::{
    io::{self, Read, Write},
    marker::PhantomData,
    mem::{align_of, size_of},
};

pub fn fill_pcm_stream_spec<C>(spec: &mut StreamSpec<C>) -> Result<(), SyphonError>
where
    C: CodecTag,
    SyphonCodec: TryInto<C>,
{
    let expected_codec = SyphonCodec::Pcm.try_into().ok();
    if expected_codec.is_some_and(|codec| spec.codec.get_or_insert(codec) != &codec) {
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

pub struct PcmCodec<T, C: CodecTag, S: Sample> {
    inner: T,
    stream_spec: StreamSpec<C>,
    signal_spec: SignalSpec,
    _sample: PhantomData<S>,
}

impl<T, C: CodecTag, S: Sample> PcmCodec<T, C, S> {
    pub fn from_stream(inner: T) -> Result<Self, SyphonError>
    where
        T: Stream<Tag = C>,
        SyphonCodec: TryInto<C>,
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
        SyphonCodec: TryInto<C>,
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

impl<T, C: CodecTag, S: Sample> Signal for PcmCodec<T, C, S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.signal_spec
    }
}

impl<T, C, S> SignalReader for PcmCodec<T, C, S>
where
    T: Read,
    C: CodecTag,
    S: Sample + ToMutByteSlice,
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

impl<T, C, S> SignalWriter for PcmCodec<T, C, S>
where
    T: Write,
    C: CodecTag,
    S: Sample + ToByteSlice,
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

impl<T, C: CodecTag, S: Sample> Stream for PcmCodec<T, C, S> {
    type Tag = C;

    fn spec(&self) -> &StreamSpec<Self::Tag> {
        &self.stream_spec
    }
}

impl<T, C, S> Read for PcmCodec<T, C, S>
where
    T: SignalReader<Sample = S>,
    C: CodecTag,
    S: Sample + FromByteSlice,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let start_i = size_of::<S>() - (buf.as_ptr() as usize % align_of::<S>());
        let aligned_len = buf.len() - start_i;
        let usable_len = aligned_len - (aligned_len % size_of::<S>());

        let sample_buf = match buf[start_i..start_i + usable_len].as_mut_slice_of::<S>() {
            Ok(buf) => buf,
            _ => return Err(io::ErrorKind::InvalidData.into()),
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

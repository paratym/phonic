use crate::{
    io::{SignalReaderRef, Stream, StreamReader, StreamSpecBuilder, StreamWriter, SignalWriterRef},
    Sample, SampleType, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError,
};
use byte_slice_cast::{AsByteSlice, AsMutByteSlice, ToByteSlice, ToMutByteSlice};

pub struct PcmCodec<T, S: Sample> {
    inner: T,
    spec: SignalSpec<S>,
}

pub fn fill_pcm_spec(spec: &mut StreamSpecBuilder) -> Result<(), SyphonError> {
    if spec.decoded_spec.block_size.is_none() {
        spec.decoded_spec.block_size = spec
            .block_size
            .zip(spec.decoded_spec.sample_type)
            .zip(spec.decoded_spec.channels)
            .map(|((b, s), c)| b / s.byte_size() / c.count() as usize)
            .or(Some(1));
    }

    let bytes_per_decoded_block = spec
        .decoded_spec
        .samples_per_block()
        .zip(spec.decoded_spec.sample_type)
        .map(|(n, s)| n * s.byte_size());

    if bytes_per_decoded_block
        .zip(spec.decoded_spec.sample_type)
        .map_or(false, |(b, s)| b % s.byte_size() != 0)
    {
        return Err(SyphonError::Unsupported);
    }

    if spec.block_size.is_none() {
        spec.block_size = bytes_per_decoded_block
    }

    if bytes_per_decoded_block
        .zip(spec.block_size)
        .map_or(false, |(d, e)| d % e != 0)
    {
        return Err(SyphonError::Unsupported);
    }

    if spec.byte_len.is_none() {
        spec.byte_len = bytes_per_decoded_block
            .zip(spec.decoded_spec.n_blocks)
            .map(|(b, n)| n * b as u64);
    } else if spec.decoded_spec.n_blocks.is_none() {
        spec.decoded_spec.n_blocks = bytes_per_decoded_block
            .zip(spec.byte_len)
            .map(|(b, n)| n / b as u64);
    }

    Ok(())
}

macro_rules! construct_pcm_signal_ref {
    ($ref:ident, $ref_inner:ident, $inner:ident, $spec:ident) => {
        Ok(match $spec.sample_type {
            SampleType::I8 => {
                (Box::new(PcmCodec::<_, i8>::new($inner, $spec.unwrap_sample_type()?))
                    as Box<dyn $ref_inner<_>>)
                    .into()
            }
            SampleType::I16 => {
                (Box::new(PcmCodec::<_, i16>::new($inner, $spec.unwrap_sample_type()?))
                    as Box<dyn $ref_inner<_>>)
                    .into()
            }
            SampleType::I32 => {
                (Box::new(PcmCodec::<_, i32>::new($inner, $spec.unwrap_sample_type()?))
                    as Box<dyn $ref_inner<_>>)
                    .into()
            }
            SampleType::I64 => {
                (Box::new(PcmCodec::<_, i64>::new($inner, $spec.unwrap_sample_type()?))
                    as Box<dyn $ref_inner<_>>)
                    .into()
            }
            SampleType::U8 => {
                (Box::new(PcmCodec::<_, u8>::new($inner, $spec.unwrap_sample_type()?))
                    as Box<dyn $ref_inner<_>>)
                    .into()
            }
            SampleType::U16 => {
                (Box::new(PcmCodec::<_, u16>::new($inner, $spec.unwrap_sample_type()?))
                    as Box<dyn $ref_inner<_>>)
                    .into()
            }
            SampleType::U32 => {
                (Box::new(PcmCodec::<_, u32>::new($inner, $spec.unwrap_sample_type()?))
                    as Box<dyn $ref_inner<_>>)
                    .into()
            }
            SampleType::U64 => {
                (Box::new(PcmCodec::<_, u64>::new($inner, $spec.unwrap_sample_type()?))
                    as Box<dyn $ref_inner<_>>)
                    .into()
            }
            SampleType::F32 => {
                (Box::new(PcmCodec::<_, f32>::new($inner, $spec.unwrap_sample_type()?))
                    as Box<dyn $ref_inner<_>>)
                    .into()
            }
            SampleType::F64 => {
                (Box::new(PcmCodec::<_, f64>::new($inner, $spec.unwrap_sample_type()?))
                    as Box<dyn $ref_inner<_>>)
                    .into()
            }
        })
    };
}

pub fn construct_pcm_signal_reader_ref<T: StreamReader + 'static>(
    inner: T,
    spec: SignalSpec<SampleType>,
) -> Result<SignalReaderRef, SyphonError> {
    construct_pcm_signal_ref!(SignalReaderRef, SignalReader, inner, spec)
}

pub fn construct_pcm_signal_writer_ref<T: StreamWriter + 'static>(
    inner: T,
    spec: SignalSpec<SampleType>,
) -> Result<SignalWriterRef, SyphonError> {
    construct_pcm_signal_ref!(SignalWriterRef, SignalWriter, inner, spec)
}

impl<T, S: Sample> PcmCodec<T, S> {
    pub fn new(inner: T, spec: SignalSpec<S>) -> Self {
        Self { inner, spec }
    }
}

impl<T, S: Sample> Signal<S> for PcmCodec<T, S> {
    fn spec(&self) -> &SignalSpec<S> {
        &self.spec
    }
}

impl<T: StreamReader, S: Sample + ToMutByteSlice> SignalReader<S> for PcmCodec<T, S> {
    fn read(&mut self, buf: &mut [S]) -> Result<usize, SyphonError> {
        let mut buf_len = buf.len();
        buf_len -= buf_len % self.spec.samples_per_block();

        let buf = buf[..buf_len].as_mut_byte_slice();
        let n_bytes = self.inner.read(buf)? * self.inner.spec().block_size;
        let bytes_per_sample = buf.len() / buf_len;
        let bytes_per_block = self.spec.samples_per_block() * bytes_per_sample;

        if n_bytes % bytes_per_block != 0 {
            todo!()
        }

        Ok(n_bytes / bytes_per_block)
    }
}

impl<T: StreamWriter, S: Sample + ToByteSlice> SignalWriter<S> for PcmCodec<T, S> {
    fn write(&mut self, buf: &[S]) -> Result<usize, SyphonError> {
        let mut buf_len = buf.len();
        buf_len -= buf_len % self.spec.samples_per_block();

        let buf = buf[..buf_len].as_byte_slice();
        let n_bytes = self.inner.write(buf)? * self.inner.spec().block_size;
        let bytes_per_sample = buf.len() / buf_len;
        let bytes_per_block = self.spec.samples_per_block() * bytes_per_sample;

        if n_bytes % bytes_per_block != 0 {
            todo!()
        }

        Ok(n_bytes / bytes_per_block)
    }
}

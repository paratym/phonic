use crate::PcmCodec;
use phonic_io_core::{
    match_tagged_signal, utils::PollIo, CodecConstructor, CodecTag, DynStream, KnownSampleType,
    StreamSpecBuilder, TaggedSignal,
};
use phonic_signal::{utils::Poll, PhonicError, PhonicResult};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PcmCodecTag;

impl CodecTag for PcmCodecTag {
    fn infer_spec(spec: &mut StreamSpecBuilder<Self>) -> PhonicResult<()> {
        infer_pcm_spec(spec)
    }
}

pub fn infer_pcm_spec<C>(spec: &mut StreamSpecBuilder<C>) -> PhonicResult<()>
where
    C: CodecTag,
    PcmCodecTag: TryInto<C>,
    PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
{
    if let Ok(expected_codec) = PcmCodecTag.try_into() {
        if *spec.codec.get_or_insert(expected_codec) != expected_codec {
            return Err(PhonicError::Unsupported);
        }
    }

    let Some(sample_type) = spec
        .sample_type
        .map(KnownSampleType::try_from)
        .transpose()?
    else {
        return Ok(());
    };

    if let Some(calculated_byte_rate) = spec
        .decoded_spec
        .sample_rate_interleaved()
        .map(|rate| rate * sample_type.byte_size() as u32)
    {
        if *spec.avg_byte_rate.get_or_insert(calculated_byte_rate) != calculated_byte_rate {
            return Err(PhonicError::Unsupported);
        }
    }

    if let Some(calculated_block_align) = spec
        .decoded_spec
        .channels
        .map(|c| c.count() as usize * sample_type.byte_size())
    {
        if *spec.block_align.get_or_insert(calculated_block_align) % calculated_block_align != 0 {
            return Err(PhonicError::Unsupported);
        }
    }

    Ok(())
}

macro_rules! dyn_construct_branch {
    ($sample:ident, $inner:ident) => {{
        // TODO: figure out why CodecConstructor needs explicit generics here
        let codec = <PcmCodec<_, _, _> as CodecConstructor<S, S::Tag>>::decoder($inner)?;
        TaggedSignal::$sample(Box::new(Poll(codec)))
    }};
}

pub fn pcm_codec_from_dyn_stream<S>(stream: S) -> PhonicResult<TaggedSignal>
where
    S: DynStream + 'static,
    PcmCodecTag: TryInto<S::Tag>,
    PhonicError: From<<PcmCodecTag as TryInto<S::Tag>>::Error>,
{
    let sample_type = KnownSampleType::try_from(stream.stream_spec().sample_type)?;
    let signal = match sample_type {
        KnownSampleType::I8 => dyn_construct_branch!(I8, stream),
        KnownSampleType::I16 => dyn_construct_branch!(I16, stream),
        KnownSampleType::I32 => dyn_construct_branch!(I32, stream),
        KnownSampleType::I64 => dyn_construct_branch!(I64, stream),

        KnownSampleType::U8 => dyn_construct_branch!(U8, stream),
        KnownSampleType::U16 => dyn_construct_branch!(U16, stream),
        KnownSampleType::U32 => dyn_construct_branch!(U32, stream),
        KnownSampleType::U64 => dyn_construct_branch!(U64, stream),

        KnownSampleType::F32 => dyn_construct_branch!(F32, stream),
        KnownSampleType::F64 => dyn_construct_branch!(F64, stream),
    };

    Ok(signal)
}

pub fn pcm_codec_from_dyn_signal<C>(
    signal: TaggedSignal,
) -> PhonicResult<Box<dyn DynStream<Tag = C>>>
where
    C: CodecTag + 'static,
    PcmCodecTag: TryInto<C>,
    PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
{
    match_tagged_signal!(signal, inner => Ok(Box::new(PollIo(PcmCodec::encoder(inner)?))))
}

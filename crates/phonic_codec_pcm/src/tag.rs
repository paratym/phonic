use crate::PcmCodec;
use phonic_core::PhonicError;
use phonic_io_core::{
    match_tagged_signal, CodecConstructor, CodecTag, DynStream, KnownSampleType, StreamSpecBuilder,
    TaggedSignal,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PcmCodecTag;

impl CodecTag for PcmCodecTag {
    fn infer_spec(spec: &mut StreamSpecBuilder<Self>) -> Result<(), PhonicError> {
        infer_pcm_spec(spec)
    }
}

pub fn infer_pcm_spec<C>(spec: &mut StreamSpecBuilder<C>) -> Result<(), PhonicError>
where
    C: CodecTag,
    PcmCodecTag: TryInto<C>,
    PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
{
    if let Ok(expected_codec) = PcmCodecTag.try_into() {
        if *spec.codec.get_or_insert(expected_codec) != expected_codec {
            return Err(PhonicError::InvalidData);
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
            return Err(PhonicError::InvalidData);
        }
    }

    if let Some(calculated_block_align) = spec
        .decoded_spec
        .channels
        .map(|c| c.count() as usize * sample_type.byte_size())
    {
        if *spec.block_align.get_or_insert(calculated_block_align) % calculated_block_align != 0 {
            return Err(PhonicError::InvalidData);
        }
    }

    Ok(())
}

pub fn pcm_codec_from_dyn_stream<S>(stream: S) -> Result<TaggedSignal, PhonicError>
where
    S: DynStream + 'static,
    PcmCodecTag: TryInto<S::Tag>,
    PhonicError: From<<PcmCodecTag as TryInto<S::Tag>>::Error>,
{
    let sample_type = KnownSampleType::try_from(stream.stream_spec().sample_type)?;

    // TODO: figure out why CodecConstructor needs explicit generics here
    let signal = match sample_type {
        KnownSampleType::I8 => {
            let codec = <PcmCodec<_, _, _> as CodecConstructor<S, S::Tag>>::decoder(stream)?;
            TaggedSignal::I8(Box::new(codec))
        }
        KnownSampleType::I16 => {
            let codec = <PcmCodec<_, _, _> as CodecConstructor<S, S::Tag>>::decoder(stream)?;
            TaggedSignal::I16(Box::new(codec))
        }
        KnownSampleType::I32 => {
            let codec = <PcmCodec<_, _, _> as CodecConstructor<S, S::Tag>>::decoder(stream)?;
            TaggedSignal::I32(Box::new(codec))
        }
        KnownSampleType::I64 => {
            let codec = <PcmCodec<_, _, _> as CodecConstructor<S, S::Tag>>::decoder(stream)?;
            TaggedSignal::I64(Box::new(codec))
        }
        KnownSampleType::U8 => {
            let codec = <PcmCodec<_, _, _> as CodecConstructor<S, S::Tag>>::decoder(stream)?;
            TaggedSignal::U8(Box::new(codec))
        }
        KnownSampleType::U16 => {
            let codec = <PcmCodec<_, _, _> as CodecConstructor<S, S::Tag>>::decoder(stream)?;
            TaggedSignal::U16(Box::new(codec))
        }
        KnownSampleType::U32 => {
            let codec = <PcmCodec<_, _, _> as CodecConstructor<S, S::Tag>>::decoder(stream)?;
            TaggedSignal::U32(Box::new(codec))
        }
        KnownSampleType::U64 => {
            let codec = <PcmCodec<_, _, _> as CodecConstructor<S, S::Tag>>::decoder(stream)?;
            TaggedSignal::U64(Box::new(codec))
        }
        KnownSampleType::F32 => {
            let codec = <PcmCodec<_, _, _> as CodecConstructor<S, S::Tag>>::decoder(stream)?;
            TaggedSignal::F32(Box::new(codec))
        }
        KnownSampleType::F64 => {
            let codec = <PcmCodec<_, _, _> as CodecConstructor<S, S::Tag>>::decoder(stream)?;
            TaggedSignal::F64(Box::new(codec))
        }
    };

    Ok(signal)
}

pub fn pcm_codec_from_dyn_signal<C>(
    signal: TaggedSignal,
) -> Result<Box<dyn DynStream<Tag = C>>, PhonicError>
where
    C: CodecTag + 'static,
    PcmCodecTag: TryInto<C>,
    PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
{
    match_tagged_signal!(signal, inner => Ok(Box::new(PcmCodec::encoder(inner)?)))
}

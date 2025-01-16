use crate::{
    codec::pcm::PcmCodec, match_tagged_signal, utils::PollIo, CodecTag, Decoder, Encoder,
    StreamSpec, StreamSpecBuilder,
};
use phonic_signal::{Channels, PhonicError, PhonicResult, SignalSpec};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PcmCodecTag;

impl PcmCodecTag {
    pub fn infer_tagged_spec<C>(spec: StreamSpecBuilder<C>) -> PhonicResult<StreamSpec<C>>
    where
        C: CodecTag,
        PcmCodecTag: TryInto<C>,
        PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
    {
        let expected_codec = PcmCodecTag.try_into()?;
        let codec = spec.codec.unwrap_or(expected_codec);
        if codec != expected_codec {
            return Err(PhonicError::InvalidInput);
        }

        let Some(sample_layout) = spec.sample_layout else {
            return Err(PhonicError::MissingData);
        };

        let sample_rate = if let Some(sample_rate) = spec.decoded_spec.sample_rate {
            sample_rate
        } else {
            let byte_rate = spec.avg_byte_rate.ok_or(PhonicError::MissingData)?;
            let n_channels = spec
                .decoded_spec
                .channels
                .ok_or(PhonicError::MissingData)?
                .count();

            byte_rate / sample_layout.size() as u32 / n_channels
        };

        let channels = if let Some(channels) = spec.decoded_spec.channels {
            channels
        } else {
            let byte_rate = spec.avg_byte_rate.ok_or(PhonicError::MissingData)?;
            let interleaved_sample_rate = byte_rate / sample_layout.size() as u32;
            if interleaved_sample_rate % sample_rate != 0 {
                return Err(PhonicError::InvalidInput);
            }

            Channels::Count(interleaved_sample_rate / sample_rate)
        };

        let calculated_byte_rate = sample_layout.size() as u32 * sample_rate * channels.count();
        let avg_byte_rate = spec.avg_byte_rate.unwrap_or(calculated_byte_rate);
        if avg_byte_rate != calculated_byte_rate {
            return Err(PhonicError::InvalidInput);
        }

        let min_block_align = sample_layout.size() * channels.count() as usize;
        let block_align = if let Some(block_align) = spec.block_align {
            if block_align % min_block_align != 0 {
                return Err(PhonicError::InvalidInput);
            }

            block_align
        } else {
            channels.count() as usize * sample_layout.size()
        };

        Ok(StreamSpec {
            codec,
            avg_byte_rate,
            block_align,
            sample_layout,
            decoded_spec: SignalSpec {
                sample_rate,
                channels,
            },
        })
    }

    #[cfg(feature = "dyn-io")]
    pub fn dyn_encoder<C: CodecTag>(
        signal: crate::dyn_io::TaggedSignal,
    ) -> PhonicResult<Box<dyn crate::dyn_io::DynStream<Tag = C>>>
    where
        PcmCodecTag: TryInto<C>,
        PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
    {
        // match_tagged_signal!(signal, inner => Ok(Box::new(PollIo(PcmCodec::encoder(inner)))))
        todo!()
    }

    #[cfg(feature = "dyn-io")]
    pub fn dyn_decoder<C: CodecTag>(
        stream: Box<dyn crate::dyn_io::DynStream<Tag = C>>,
    ) -> PhonicResult<crate::dyn_io::TaggedSignal> {
        todo!()
        // use crate::dyn_io::{KnownSampleType, TaggedSignal};
        //
        // macro_rules! dyn_construct_branch {
        //     ($sample:ident, $inner:ident) => {{
        //         // TODO: figure out why CodecConstructor needs explicit generics here
        //         let codec = <PcmCodec<_, _, _> as Decoder<S, S::Tag>>::decoder($inner)?;
        //         TaggedSignal::$sample(Box::new(Poll(codec)))
        //     }};
        // }
        //
        // let sample_type = stream.stream_spec().sample_layout.id().try_into()?;
        // let signal = match sample_type {
        //     KnownSampleType::I8 => dyn_construct_branch!(I8, stream),
        //     KnownSampleType::I16 => dyn_construct_branch!(I16, stream),
        //     KnownSampleType::I32 => dyn_construct_branch!(I32, stream),
        //     KnownSampleType::I64 => dyn_construct_branch!(I64, stream),
        //
        //     KnownSampleType::U8 => dyn_construct_branch!(U8, stream),
        //     KnownSampleType::U16 => dyn_construct_branch!(U16, stream),
        //     KnownSampleType::U32 => dyn_construct_branch!(U32, stream),
        //     KnownSampleType::U64 => dyn_construct_branch!(U64, stream),
        //
        //     KnownSampleType::F32 => dyn_construct_branch!(F32, stream),
        //     KnownSampleType::F64 => dyn_construct_branch!(F64, stream),
        // };
        //
        // Ok(signal)
    }
}

impl CodecTag for PcmCodecTag {
    fn infer_spec(spec: StreamSpecBuilder<Self>) -> PhonicResult<StreamSpec<Self>> {
        PcmCodecTag::infer_tagged_spec(spec)
    }
}

#[cfg(feature = "dyn-io")]
impl From<PcmCodecTag> for crate::dyn_io::KnownCodec {
    fn from(_: PcmCodecTag) -> Self {
        Self::Pcm
    }
}

#[cfg(feature = "dyn-io")]
impl TryFrom<crate::dyn_io::KnownCodec> for PcmCodecTag {
    type Error = PhonicError;

    fn try_from(codec: crate::dyn_io::KnownCodec) -> Result<Self, Self::Error> {
        match codec {
            crate::dyn_io::KnownCodec::Pcm => Ok(Self),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

//
// pub fn pcm_codec_from_dyn_signal<C>(
//     signal: TaggedSignal,
// ) -> PhonicResult<Box<dyn DynStream<Tag = C>>>
// where
//     C: CodecTag + 'static,
//     PcmCodecTag: TryInto<C>,
//     PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
// {
//     match_tagged_signal!(signal, inner => Ok(Box::new(PollIo(PcmCodec::encoder(inner)?))))
// }

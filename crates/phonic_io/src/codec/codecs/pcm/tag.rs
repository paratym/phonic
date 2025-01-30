use crate::{
    codecs::pcm::{Endianess, PcmCodec},
    utils::{PollIo, UnWriteable},
    CodecFromSignal, CodecFromStream, CodecTag, StreamSpec, StreamSpecBuilder,
};
use phonic_signal::{utils::Poll, PhonicError, PhonicResult, SignalSpec};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum PcmCodecTag {
    LE,
    BE,
}

impl PcmCodecTag {
    pub fn infer_tagged_spec<C>(spec: StreamSpecBuilder<C>) -> PhonicResult<StreamSpec<C>>
    where
        C: CodecTag + TryInto<PcmCodecTag>,
        PcmCodecTag: TryInto<C>,
        PhonicError: From<<C as TryInto<PcmCodecTag>>::Error>,
        PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
    {
        let codec = spec
            .codec
            .map(TryInto::<PcmCodecTag>::try_into)
            .transpose()?
            .unwrap_or_default()
            .try_into()?;

        let Some(sample_layout) = spec.sample else {
            return Err(PhonicError::missing_data());
        };

        let sample_rate = if let Some(sample_rate) = spec.decoded.sample_rate {
            sample_rate
        } else {
            let byte_rate = spec.byte_rate.ok_or(PhonicError::missing_data())?;
            let n_channels = spec.decoded.n_channels.ok_or(PhonicError::missing_data())?;

            byte_rate as usize / sample_layout.size() / n_channels
        };

        let n_channels = if let Some(channels) = spec.decoded.n_channels {
            channels
        } else {
            let byte_rate = spec.byte_rate.ok_or(PhonicError::missing_data())?;
            let interleaved_sample_rate = byte_rate as usize / sample_layout.size();
            if interleaved_sample_rate % sample_rate != 0 {
                return Err(PhonicError::invalid_input());
            }

            interleaved_sample_rate / sample_rate
        };

        let calculated_byte_rate = sample_layout.size() * sample_rate * n_channels;
        let avg_byte_rate = spec.byte_rate.unwrap_or(calculated_byte_rate);
        if avg_byte_rate != calculated_byte_rate {
            return Err(PhonicError::invalid_input());
        }

        let min_block_align = sample_layout.size() * n_channels;
        let block_align = if let Some(block_align) = spec.block_align {
            if block_align % min_block_align != 0 {
                return Err(PhonicError::invalid_input());
            }

            block_align
        } else {
            n_channels * sample_layout.size()
        };

        Ok(StreamSpec {
            codec,
            byte_rate: avg_byte_rate,
            block_align,
            sample: sample_layout,
            decoded: SignalSpec {
                sample_rate,
                n_channels,
            },
        })
    }

    #[cfg(feature = "dynamic")]
    pub fn from_dyn_signal<C>(
        tag: C,
        signal: crate::dynamic::TaggedSignal,
    ) -> PhonicResult<Box<dyn crate::dynamic::DynStream<Tag = C>>>
    where
        C: CodecTag + TryInto<PcmCodecTag> + 'static,
        PcmCodecTag: TryInto<C>,
        PhonicError: From<<C as TryInto<PcmCodecTag>>::Error>,
        PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
    {
        crate::match_tagged_signal!(signal, inner => Ok(Box::new(PollIo(UnWriteable(PcmCodec::from_signal(tag, inner)?)))))
    }

    #[cfg(feature = "dynamic")]
    pub fn from_dyn_stream<C>(
        stream: Box<dyn crate::dynamic::DynStream<Tag = C>>,
    ) -> PhonicResult<crate::dynamic::TaggedSignal>
    where
        C: CodecTag + TryInto<PcmCodecTag> + 'static,
        PcmCodecTag: TryInto<C>,
        PhonicError: From<<C as TryInto<PcmCodecTag>>::Error>,
        PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
    {
        use crate::dynamic::{KnownSampleType, TaggedSignal};

        let sample_type = KnownSampleType::try_from(stream.stream_spec().sample.id())?;
        let signal = match sample_type {
            KnownSampleType::I8 => TaggedSignal::I8(Box::new(Poll(PcmCodec::from_stream(stream)?))),
            KnownSampleType::I16 => {
                TaggedSignal::I16(Box::new(Poll(PcmCodec::from_stream(stream)?)))
            }
            KnownSampleType::I32 => {
                TaggedSignal::I32(Box::new(Poll(PcmCodec::from_stream(stream)?)))
            }
            KnownSampleType::I64 => {
                TaggedSignal::I64(Box::new(Poll(PcmCodec::from_stream(stream)?)))
            }
            KnownSampleType::U8 => TaggedSignal::U8(Box::new(Poll(PcmCodec::from_stream(stream)?))),
            KnownSampleType::U16 => {
                TaggedSignal::U16(Box::new(Poll(PcmCodec::from_stream(stream)?)))
            }
            KnownSampleType::U32 => {
                TaggedSignal::U32(Box::new(Poll(PcmCodec::from_stream(stream)?)))
            }
            KnownSampleType::U64 => {
                TaggedSignal::U64(Box::new(Poll(PcmCodec::from_stream(stream)?)))
            }
            KnownSampleType::F32 => {
                TaggedSignal::F32(Box::new(Poll(PcmCodec::from_stream(stream)?)))
            }
            KnownSampleType::F64 => {
                TaggedSignal::F64(Box::new(Poll(PcmCodec::from_stream(stream)?)))
            }
        };

        Ok(signal)
    }
}

impl Default for PcmCodecTag {
    fn default() -> Self {
        Self::from(Endianess::default())
    }
}

impl CodecTag for PcmCodecTag {
    fn infer_spec(spec: StreamSpecBuilder<Self>) -> PhonicResult<StreamSpec<Self>> {
        PcmCodecTag::infer_tagged_spec(spec)
    }
}

#[cfg(feature = "dynamic")]
impl From<PcmCodecTag> for crate::dynamic::KnownCodec {
    fn from(tag: PcmCodecTag) -> Self {
        use crate::dynamic::KnownCodec;

        match tag {
            PcmCodecTag::LE => KnownCodec::PcmLE,
            PcmCodecTag::BE => KnownCodec::PcmBE,
        }
    }
}

#[cfg(feature = "dynamic")]
impl From<crate::dynamic::KnownCodec> for Option<PcmCodecTag> {
    fn from(codec: crate::dynamic::KnownCodec) -> Self {
        use crate::dynamic::KnownCodec;

        match codec {
            KnownCodec::PcmLE => Some(PcmCodecTag::LE),
            KnownCodec::PcmBE => Some(PcmCodecTag::BE),

            #[allow(unreachable_patterns)]
            _ => None,
        }
    }
}

#[cfg(feature = "dynamic")]
impl TryFrom<crate::dynamic::KnownCodec> for PcmCodecTag {
    type Error = PhonicError;

    fn try_from(codec: crate::dynamic::KnownCodec) -> Result<Self, Self::Error> {
        Option::<Self>::from(codec).ok_or(PhonicError::unsupported())
    }
}

#[cfg(test)]
mod tests {
    use crate::{codecs::pcm::PcmCodecTag, StreamSpec};
    use phonic_signal::PhonicError;

    #[test]
    fn infer_can_fill_any_single_missing_data_point() {
        todo!()
    }

    #[test]
    fn infer_properly_rejects_missing_data() {
        macro_rules! impl_test {
            (
                $spec:ident = $specDef:expr;
                $mutLayout:stmt;
                $mutSampleRate:stmt;
                $mutChannels:stmt;
                $mutByteRate:stmt;
            ) => {
                impl_test!($spec = $specDef; $mutLayout);
                impl_test!($spec = $specDef; $mutSampleRate);
                impl_test!($spec = $specDef; $mutChannels);
                impl_test!($spec = $specDef; $mutByteRate);

                impl_test!(
                    $spec = $specDef;
                    $mutLayout;
                    $mutSampleRate
                );

                impl_test!(
                    $spec = $specDef;
                    $mutLayout;
                    $mutChannels
                );

                impl_test!(
                    $spec = $specDef;
                    $mutLayout;
                    $mutByteRate
                );


                impl_test!(
                    $spec = $specDef;
                    $mutSampleRate;
                    $mutChannels
                );

                impl_test!(
                    $spec = $specDef;
                    $mutSampleRate;
                    $mutByteRate
                );

                impl_test!(
                    $spec = $specDef;
                    $mutChannels;
                    $mutByteRate
                );
            };
            (
                $spec:ident = $specDef:expr;
                $($mutation:stmt);*
            ) => {
                let mut $spec = $specDef;
                $($mutation)*
                match PcmCodecTag::infer_tagged_spec($spec) {
                    Err(PhonicError::MissingData { .. }) => (),
                    result => panic!("expected PhonicError::MissingData found: {result:?}"),
                }
            }
        }

        impl_test!(
                spec = StreamSpec::<PcmCodecTag>::builder();
                spec = spec.with_sample_type::<f32>();
                spec.decoded.sample_rate = Some(48000);
                spec.decoded.n_channels = Some(2);
                spec.byte_rate = Some(4 * 48000 * 2);
        );
    }

    #[test]
    fn infer_properly_rejects_invalid_data() {
        todo!()
    }

    #[cfg(feature = "dynamic")]
    mod dyn_construct {
        macro_rules! impl_test {
            ($name:ident, $sample:ty, $tag:ident) => {
                #[test]
                fn $name() {
                    let spec = SignalSpec {
                        sample_rate: 48000,
                        n_channels: 2,
                    };

                    let signal = Poll(Infinite(UnSeekable(Indexed::new(
                        NullSignal::<$sample>::new(spec),
                    ))));

                    let dyn_signal = TaggedSignal::from(
                        Box::new(signal) as Box<dyn DynSignal<Sample = $sample>>
                    );

                    let signal_codec = PcmCodecTag::from_dyn_signal(PcmCodecTag::$tag, dyn_signal)
                        .expect("failed to construct signal codec");

                    assert_eq!(
                        signal_codec.stream_spec().sample.id(),
                        TypeId::of::<$sample>()
                    );

                    let stream_codec = PcmCodecTag::from_dyn_stream(signal_codec)
                        .expect("failed to construct stream codec");

                    assert_eq!(stream_codec.sample_type().id(), TypeId::of::<$sample>())
                }
            };
            ($name:ident, $sample:ty) => {
                mod $name {
                    use crate::{
                        codecs::pcm::PcmCodecTag,
                        dynamic::{DynSignal, TaggedSignal},
                        utils::{Infinite, UnSeekable},
                    };
                    use phonic_signal::{
                        utils::{Indexed, NullSignal, Poll},
                        SignalSpec,
                    };
                    use std::any::TypeId;

                    impl_test!(little_endian, $sample, LE);
                    impl_test!(big_endian, $sample, BE);
                }
            };
        }

        impl_test!(from_u8, u8);
        impl_test!(from_u16, u16);
        impl_test!(from_u32, u32);
        impl_test!(from_u64, u64);

        impl_test!(from_i8, i8);
        impl_test!(from_i16, i16);
        impl_test!(from_i32, i32);
        impl_test!(from_i64, i64);

        impl_test!(from_f32, f32);
        impl_test!(from_f64, f64);
    }
}

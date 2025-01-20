use crate::{
    codec::pcm::{Endianess, PcmCodec},
    utils::{PollIo, UnWriteable},
    CodecFromSignal, CodecFromStream, CodecTag, StreamSpec, StreamSpecBuilder,
};
use phonic_signal::{utils::Poll, Channels, PhonicError, PhonicResult, SignalSpec};

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
    pub fn from_dyn_signal<C>(
        tag: C,
        signal: crate::dyn_io::TaggedSignal,
    ) -> PhonicResult<Box<dyn crate::dyn_io::DynStream<Tag = C>>>
    where
        C: CodecTag + TryInto<PcmCodecTag> + 'static,
        PcmCodecTag: TryInto<C>,
        PhonicError: From<<C as TryInto<PcmCodecTag>>::Error>,
        PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
    {
        crate::match_tagged_signal!(signal, inner => Ok(Box::new(PollIo(UnWriteable(PcmCodec::from_signal(tag, inner)?)))))
    }

    #[cfg(feature = "dyn-io")]
    pub fn from_dyn_stream<C>(
        stream: Box<dyn crate::dyn_io::DynStream<Tag = C>>,
    ) -> PhonicResult<crate::dyn_io::TaggedSignal>
    where
        C: CodecTag + TryInto<PcmCodecTag> + 'static,
        PcmCodecTag: TryInto<C>,
        PhonicError: From<<C as TryInto<PcmCodecTag>>::Error>,
        PhonicError: From<<PcmCodecTag as TryInto<C>>::Error>,
    {
        use crate::dyn_io::{KnownSampleType, TaggedSignal};

        let sample_type = KnownSampleType::try_from(stream.stream_spec().sample_layout.id())?;
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

#[cfg(feature = "dyn-io")]
impl From<PcmCodecTag> for crate::dyn_io::KnownCodec {
    fn from(tag: PcmCodecTag) -> Self {
        use crate::dyn_io::KnownCodec;

        match tag {
            PcmCodecTag::LE => KnownCodec::PcmLE,
            PcmCodecTag::BE => KnownCodec::PcmBE,
        }
    }
}

#[cfg(feature = "dyn-io")]
impl TryFrom<crate::dyn_io::KnownCodec> for PcmCodecTag {
    type Error = PhonicError;

    fn try_from(codec: crate::dyn_io::KnownCodec) -> Result<Self, Self::Error> {
        use crate::dyn_io::KnownCodec;

        match codec {
            KnownCodec::PcmLE => Ok(Self::LE),
            KnownCodec::PcmBE => Ok(Self::BE),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{codec::pcm::PcmCodecTag, StreamSpec};
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
                    Err(PhonicError::MissingData) => (),
                    result => panic!("expected PhonicError::MissingData found: {result:?}"),
                }
            }
        }

        impl_test!(
                spec = StreamSpec::<PcmCodecTag>::builder();
                spec = spec.with_sample_type::<f32>();
                spec.decoded_spec.sample_rate = Some(48000);
                spec.decoded_spec.channels = Some(2.into());
                spec.avg_byte_rate = Some(4 * 48000 * 2);
        );
    }

    #[test]
    fn infer_properly_rejects_invalid_data() {
        todo!()
    }

    #[cfg(feature = "dyn-io")]
    mod dyn_construct {
        macro_rules! impl_test {
            ($name:ident, $sample:ty, $tag:ident) => {
                #[test]
                fn $name() {
                    let spec = SignalSpec {
                        sample_rate: 48000,
                        channels: 2.into(),
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
                        signal_codec.stream_spec().sample_layout.id(),
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
                        codec::pcm::PcmCodecTag,
                        dyn_io::{DynSignal, TaggedSignal},
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

use phonic_core::PhonicError;
use phonic_io_core::{CodecTag, DynCodecConstructor, DynStream, StreamSpecBuilder, TaggedSignal};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum KnownCodec {
    #[cfg(feature = "pcm")]
    Pcm,
}

impl CodecTag for KnownCodec {
    fn infer_spec(spec: &mut StreamSpecBuilder<Self>) -> Result<(), PhonicError> {
        use crate::codecs::*;

        match spec.codec {
            #[cfg(feature = "pcm")]
            Some(Self::Pcm) => pcm::infer_pcm_spec(spec),

            _ => Ok(()),
        }
    }
}

impl DynCodecConstructor for KnownCodec {
    fn encoder(&self, signal: TaggedSignal) -> Result<Box<dyn DynStream<Tag = Self>>, PhonicError> {
        use crate::codecs::*;

        match self {
            #[cfg(feature = "pcm")]
            Self::Pcm => pcm::pcm_codec_from_dyn_signal(signal),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }

    fn decoder<S: DynStream<Tag = Self> + 'static>(stream: S) -> Result<TaggedSignal, PhonicError> {
        use crate::codecs::*;

        match stream.stream_spec().codec {
            #[cfg(feature = "pcm")]
            Self::Pcm => pcm::pcm_codec_from_dyn_stream(stream),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

#[cfg(feature = "wave")]
impl TryFrom<crate::formats::wave::WaveSupportedCodec> for KnownCodec {
    type Error = PhonicError;

    fn try_from(codec: crate::formats::wave::WaveSupportedCodec) -> Result<Self, Self::Error> {
        match codec {
            #[cfg(feature = "pcm")]
            crate::formats::wave::WaveSupportedCodec::Pcm => Ok(Self::Pcm),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

#[cfg(feature = "wave")]
impl TryFrom<KnownCodec> for crate::formats::wave::WaveSupportedCodec {
    type Error = PhonicError;

    fn try_from(codec: KnownCodec) -> Result<Self, Self::Error> {
        match codec {
            #[cfg(feature = "pcm")]
            KnownCodec::Pcm => Ok(Self::Pcm),
            _ => Err(PhonicError::Unsupported),
        }
    }
}

#[cfg(feature = "pcm")]
impl From<crate::codecs::pcm::PcmCodecTag> for KnownCodec {
    fn from(_: crate::codecs::pcm::PcmCodecTag) -> Self {
        Self::Pcm
    }
}

#[cfg(feature = "pcm")]
impl TryFrom<KnownCodec> for crate::codecs::pcm::PcmCodecTag {
    type Error = PhonicError;

    fn try_from(codec: KnownCodec) -> Result<Self, Self::Error> {
        match codec {
            KnownCodec::Pcm => Ok(Self),
            _ => Err(PhonicError::Unsupported),
        }
    }
}

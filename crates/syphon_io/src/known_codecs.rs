use std::hash::Hash;
use syphon_core::SyphonError;
use syphon_io_core::{utils::TaggedSignal, CodecTag, DynCodecConstructor, DynStream, StreamSpec};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum KnownCodec {
    #[cfg(feature = "pcm")]
    Pcm,
}

impl CodecTag for KnownCodec {
    fn fill_spec(spec: &mut StreamSpec<Self>) -> Result<(), SyphonError> {
        match spec.codec {
            #[cfg(feature = "pcm")]
            Some(Self::Pcm) => crate::codecs::pcm::fill_pcm_spec(spec),

            _ => Ok(()),
        }
    }
}

impl DynCodecConstructor for KnownCodec {
    fn from_signal(
        &self,
        signal: TaggedSignal,
    ) -> Result<Box<dyn DynStream<Tag = Self>>, SyphonError> {
        match self {
            #[cfg(feature = "pcm")]
            Self::Pcm => crate::codecs::pcm::pcm_codec_from_signal(signal),
        }
    }

    fn from_stream<S: DynStream<Tag = Self> + 'static>(
        stream: S,
    ) -> Result<TaggedSignal, SyphonError> {
        match stream.spec().codec {
            #[cfg(feature = "pcm")]
            Some(Self::Pcm) => crate::codecs::pcm::pcm_codec_from_stream(stream),

            None => Err(SyphonError::MissingData),
            _ => Err(SyphonError::Unsupported),
        }
    }
}

#[cfg(feature = "wave")]
impl From<crate::formats::wave::WaveSupportedCodec> for KnownCodec {
    fn from(codec: crate::formats::wave::WaveSupportedCodec) -> Self {
        match codec {
            crate::formats::wave::WaveSupportedCodec::Pcm => Self::Pcm,
        }
    }
}

#[cfg(feature = "wave")]
impl TryFrom<KnownCodec> for crate::formats::wave::WaveSupportedCodec {
    type Error = SyphonError;

    fn try_from(codec: KnownCodec) -> Result<Self, Self::Error> {
        match codec {
            KnownCodec::Pcm => Ok(Self::Pcm),
            _ => Err(SyphonError::Unsupported),
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
    type Error = SyphonError;

    fn try_from(codec: KnownCodec) -> Result<Self, Self::Error> {
        match codec {
            KnownCodec::Pcm => Ok(Self),
            _ => Err(SyphonError::Unsupported),
        }
    }
}

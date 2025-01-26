use crate::{CodecTag, FormatTag, StreamSpec, StreamSpecBuilder};
use phonic_signal::{PhonicError, PhonicResult};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct WaveFormatTag;

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum WaveSupportedCodec {
    #[cfg(feature = "pcm")]
    PcmLE,
}

impl FormatTag for WaveFormatTag {
    type Codec = WaveSupportedCodec;
}

impl CodecTag for WaveSupportedCodec {
    fn infer_spec(spec: StreamSpecBuilder<Self>) -> PhonicResult<StreamSpec<Self>> {
        match spec.codec {
            #[cfg(feature = "pcm")]
            Some(Self::PcmLE) => crate::codecs::pcm::PcmCodecTag::infer_tagged_spec(spec),

            None => Err(PhonicError::MissingData),
        }
    }
}

impl Default for WaveSupportedCodec {
    fn default() -> Self {
        Self::PcmLE
    }
}

#[cfg(feature = "pcm")]
impl TryFrom<crate::codecs::pcm::PcmCodecTag> for WaveSupportedCodec {
    type Error = PhonicError;

    fn try_from(tag: crate::codecs::pcm::PcmCodecTag) -> Result<Self, Self::Error> {
        use crate::codecs::pcm::PcmCodecTag;

        match tag {
            PcmCodecTag::LE => Ok(Self::PcmLE),
            PcmCodecTag::BE => Err(PhonicError::Unsupported),
        }
    }
}

#[cfg(feature = "pcm")]
impl TryFrom<WaveSupportedCodec> for crate::codecs::pcm::PcmCodecTag {
    type Error = PhonicError;

    fn try_from(codec: WaveSupportedCodec) -> Result<Self, Self::Error> {
        match codec {
            WaveSupportedCodec::PcmLE => Ok(Self::LE),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

#[cfg(feature = "dynamic")]
impl From<WaveFormatTag> for crate::dynamic::KnownFormat {
    fn from(tag: WaveFormatTag) -> Self {
        match tag {
            WaveFormatTag => Self::Wave,
        }
    }
}

#[cfg(feature = "dynamic")]
impl TryFrom<crate::dynamic::KnownFormat> for WaveFormatTag {
    type Error = PhonicError;

    fn try_from(format: crate::dynamic::KnownFormat) -> Result<Self, Self::Error> {
        match format {
            crate::dynamic::KnownFormat::Wave => Ok(Self),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

#[cfg(feature = "dynamic")]
impl TryFrom<WaveSupportedCodec> for crate::dynamic::KnownCodec {
    type Error = PhonicError;

    fn try_from(codec: crate::formats::wave::WaveSupportedCodec) -> Result<Self, Self::Error> {
        match codec {
            #[cfg(feature = "pcm")]
            crate::formats::wave::WaveSupportedCodec::PcmLE => Ok(Self::PcmLE),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

#[cfg(feature = "dynamic")]
impl TryFrom<crate::dynamic::KnownCodec> for WaveSupportedCodec {
    type Error = PhonicError;

    fn try_from(codec: crate::dynamic::KnownCodec) -> Result<Self, Self::Error> {
        match codec {
            #[cfg(feature = "pcm")]
            crate::dynamic::KnownCodec::PcmLE => Ok(Self::PcmLE),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

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

            None => Err(PhonicError::missing_data()),
        }
    }
}

impl Default for WaveSupportedCodec {
    fn default() -> Self {
        Self::PcmLE
    }
}

#[cfg(feature = "pcm")]
impl From<crate::codecs::pcm::PcmCodecTag> for Option<WaveSupportedCodec> {
    fn from(codec: crate::codecs::pcm::PcmCodecTag) -> Self {
        use crate::codecs::pcm::PcmCodecTag;

        match codec {
            PcmCodecTag::LE => Some(WaveSupportedCodec::PcmLE),
            PcmCodecTag::BE => None,
        }
    }
}

#[cfg(feature = "pcm")]
impl TryFrom<crate::codecs::pcm::PcmCodecTag> for WaveSupportedCodec {
    type Error = PhonicError;

    fn try_from(codec: crate::codecs::pcm::PcmCodecTag) -> Result<Self, Self::Error> {
        Option::<Self>::from(codec).ok_or(PhonicError::unsupported())
    }
}

#[cfg(feature = "pcm")]
impl From<WaveSupportedCodec> for Option<crate::codecs::pcm::PcmCodecTag> {
    fn from(codec: WaveSupportedCodec) -> Self {
        match codec {
            WaveSupportedCodec::PcmLE => Some(crate::codecs::pcm::PcmCodecTag::LE),

            #[allow(unreachable_patterns)]
            _ => None,
        }
    }
}

#[cfg(feature = "pcm")]
impl TryFrom<WaveSupportedCodec> for crate::codecs::pcm::PcmCodecTag {
    type Error = PhonicError;

    fn try_from(codec: WaveSupportedCodec) -> Result<Self, Self::Error> {
        Option::<Self>::from(codec).ok_or(PhonicError::unsupported())
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
impl From<crate::dynamic::KnownFormat> for Option<WaveFormatTag> {
    fn from(format: crate::dynamic::KnownFormat) -> Self {
        match format {
            crate::dynamic::KnownFormat::Wave => Some(WaveFormatTag),

            #[allow(unreachable_patterns)]
            _ => None,
        }
    }
}

#[cfg(feature = "dynamic")]
impl TryFrom<crate::dynamic::KnownFormat> for WaveFormatTag {
    type Error = PhonicError;

    fn try_from(format: crate::dynamic::KnownFormat) -> Result<Self, Self::Error> {
        Option::<Self>::from(format).ok_or(PhonicError::unsupported())
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
            _ => Err(PhonicError::unsupported()),
        }
    }
}

#[cfg(feature = "dynamic")]
impl From<crate::dynamic::KnownCodec> for Option<WaveSupportedCodec> {
    fn from(codec: crate::dynamic::KnownCodec) -> Self {
        match codec {
            #[cfg(feature = "pcm")]
            crate::dynamic::KnownCodec::PcmLE => Some(WaveSupportedCodec::PcmLE),

            #[allow(unreachable_patterns)]
            _ => None,
        }
    }
}

#[cfg(feature = "dynamic")]
impl TryFrom<crate::dynamic::KnownCodec> for WaveSupportedCodec {
    type Error = PhonicError;

    fn try_from(codec: crate::dynamic::KnownCodec) -> Result<Self, Self::Error> {
        Option::<Self>::from(codec).ok_or(PhonicError::unsupported())
    }
}

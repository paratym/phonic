use crate::{utils::FormatIdentifiers, CodecTag, FormatTag, StreamSpec, StreamSpecBuilder};
use phonic_signal::{PhonicError, PhonicResult};

pub static WAVE_IDENTIFIERS: FormatIdentifiers = FormatIdentifiers {
    file_extensions: &["wav", "wave"],
    mime_types: &["audio/vnd.wave", "audio/x-wav", "audio/wav", "audio/wave"],
    markers: &[b"RIFF", b"WAVE"],
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct WaveFormatTag;

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum WaveSupportedCodec {
    #[cfg(feature = "pcm")]
    Pcm,
}

impl FormatTag for WaveFormatTag {
    type Codec = WaveSupportedCodec;
}

impl CodecTag for WaveSupportedCodec {
    fn infer_spec(spec: StreamSpecBuilder<Self>) -> PhonicResult<StreamSpec<Self>> {
        match spec.codec {
            #[cfg(feature = "pcm")]
            Some(Self::Pcm) => crate::codec::pcm::PcmCodecTag::infer_tagged_spec(spec),

            None => Err(PhonicError::MissingData),
        }
    }
}

#[cfg(feature = "pcm")]
impl From<crate::codec::pcm::PcmCodecTag> for WaveSupportedCodec {
    fn from(tag: crate::codec::pcm::PcmCodecTag) -> Self {
        match tag {
            crate::codec::pcm::PcmCodecTag => Self::Pcm,
        }
    }
}

#[cfg(feature = "pcm")]
impl TryFrom<WaveSupportedCodec> for crate::codec::pcm::PcmCodecTag {
    type Error = PhonicError;

    fn try_from(codec: WaveSupportedCodec) -> Result<Self, Self::Error> {
        match codec {
            WaveSupportedCodec::Pcm => Ok(Self),
        }
    }
}

#[cfg(feature = "dyn-io")]
impl From<WaveFormatTag> for crate::dyn_io::KnownFormat {
    fn from(tag: WaveFormatTag) -> Self {
        match tag {
            WaveFormatTag => Self::Wave,
        }
    }
}

#[cfg(feature = "dyn-io")]
impl TryFrom<crate::dyn_io::KnownFormat> for WaveFormatTag {
    type Error = PhonicError;

    fn try_from(format: crate::dyn_io::KnownFormat) -> Result<Self, Self::Error> {
        match format {
            crate::dyn_io::KnownFormat::Wave => Ok(Self),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

#[cfg(feature = "dyn-io")]
impl TryFrom<WaveSupportedCodec> for crate::dyn_io::KnownCodec {
    type Error = PhonicError;

    fn try_from(codec: crate::format::wave::WaveSupportedCodec) -> Result<Self, Self::Error> {
        match codec {
            #[cfg(feature = "pcm")]
            crate::format::wave::WaveSupportedCodec::Pcm => Ok(Self::Pcm),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

#[cfg(feature = "dyn-io")]
impl TryFrom<crate::dyn_io::KnownCodec> for WaveSupportedCodec {
    type Error = PhonicError;

    fn try_from(codec: crate::dyn_io::KnownCodec) -> Result<Self, Self::Error> {
        match codec {
            #[cfg(feature = "pcm")]
            crate::dyn_io::KnownCodec::Pcm => Ok(Self::Pcm),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

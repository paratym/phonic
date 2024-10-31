use phonic_core::PhonicError;
use phonic_io_core::{utils::FormatIdentifiers, CodecTag, FormatTag, StreamSpecBuilder};

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
    fn infer_spec(spec: &mut StreamSpecBuilder<Self>) -> Result<(), PhonicError> {
        match spec.codec {
            #[cfg(feature = "pcm")]
            Some(Self::Pcm) => phonic_codec_pcm::infer_pcm_spec(spec),

            None => Ok(()),
        }
    }
}

#[cfg(feature = "pcm")]
impl From<phonic_codec_pcm::PcmCodecTag> for WaveSupportedCodec {
    fn from(_: phonic_codec_pcm::PcmCodecTag) -> Self {
        Self::Pcm
    }
}

#[cfg(feature = "pcm")]
impl TryFrom<WaveSupportedCodec> for phonic_codec_pcm::PcmCodecTag {
    type Error = PhonicError;

    fn try_from(codec: WaveSupportedCodec) -> Result<Self, Self::Error> {
        match codec {
            WaveSupportedCodec::Pcm => Ok(Self),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

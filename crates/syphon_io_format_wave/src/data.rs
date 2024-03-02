use syphon_core::SyphonError;
use syphon_io_codec_pcm::PcmCodecTag;
use syphon_io_core::{
    CodecRegistry, CodecTag, FormatData, FormatIdentifiers, FormatTag, StreamSpec,
};

pub static WAVE_IDENTIFIERS: FormatIdentifiers = FormatIdentifiers {
    file_extensions: &["wav", "wave"],
    mime_types: &["audio/vnd.wave", "audio/x-wav", "audio/wav", "audio/wave"],
    markers: &[b"RIFF", b"WAVE"],
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct WaveFormatTag();

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SupportedWaveCodec {
    Pcm,
}

impl FormatTag for WaveFormatTag {
    type Codec = SupportedWaveCodec;
}

impl CodecTag for SupportedWaveCodec {}

impl From<PcmCodecTag> for SupportedWaveCodec {
    fn from(_: PcmCodecTag) -> Self {
        Self::Pcm
    }
}

impl TryFrom<SupportedWaveCodec> for PcmCodecTag {
    type Error = SyphonError;

    fn try_from(codec: SupportedWaveCodec) -> Result<Self, Self::Error> {
        match codec {
            SupportedWaveCodec::Pcm => Ok(PcmCodecTag()),
            _ => Err(SyphonError::Unsupported),
        }
    }
}

pub fn fill_wave_format_data<F>(data: &mut FormatData<F>) -> Result<(), SyphonError>
where
    F: FormatTag,
    WaveFormatTag: Into<F>,
    F::Codec: CodecRegistry,
    SupportedWaveCodec: Into<F::Codec>,
{
    let expected_format = WaveFormatTag().into();
    if data.format.get_or_insert(expected_format) != &expected_format || data.streams.len() > 1 {
        return Err(SyphonError::InvalidData);
    }

    if data.streams.is_empty() {
        data.streams.push(StreamSpec::new());
    }

    let stream_spec = data.streams.first_mut().ok_or(SyphonError::InvalidData)?;
    let expected_codec = SupportedWaveCodec::Pcm.into();
    if stream_spec.codec.get_or_insert(expected_codec) != &expected_codec {
        return Err(SyphonError::InvalidData);
    }

    stream_spec.fill()?;
    Ok(())
}

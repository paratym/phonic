use syphon_codec_pcm::PcmCodecTag;
use syphon_core::SyphonError;
use syphon_io_core::{utils::FormatIdentifiers, CodecTag, FormatData, FormatTag, StreamSpec};

pub static WAVE_IDENTIFIERS: FormatIdentifiers = FormatIdentifiers {
    file_extensions: &["wav", "wave"],
    mime_types: &["audio/vnd.wave", "audio/x-wav", "audio/wav", "audio/wave"],
    markers: &[b"RIFF", b"WAVE"],
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct WaveFormatTag;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WaveSupportedCodec {
    Pcm,
}

pub fn fill_wave_data<F>(data: &mut FormatData<F>) -> Result<(), SyphonError>
where
    F: FormatTag,
    WaveFormatTag: TryInto<F>,
{
    let expected_format = WaveFormatTag.try_into().ok();
    if data.format.is_some() && data.format != expected_format {
        return Err(SyphonError::InvalidData);
    } else {
        data.format = expected_format;
    }

    match data.streams.len() {
        0 => data.streams.push(StreamSpec::new()),
        1 => data.streams.first_mut().unwrap().fill()?,
        _ => return Err(SyphonError::Unsupported),
    }

    Ok(())
}

impl FormatTag for WaveFormatTag {
    type Codec = WaveSupportedCodec;

    fn fill_data(data: &mut FormatData<Self>) -> Result<(), SyphonError> {
        fill_wave_data(data)
    }
}

impl CodecTag for WaveSupportedCodec {
    fn fill_spec(_: &mut StreamSpec<Self>) -> Result<(), SyphonError> {
        Err(SyphonError::Unsupported)
    }
}

impl From<PcmCodecTag> for WaveSupportedCodec {
    fn from(_: PcmCodecTag) -> Self {
        Self::Pcm
    }
}

impl TryFrom<WaveSupportedCodec> for PcmCodecTag {
    type Error = SyphonError;

    fn try_from(codec: WaveSupportedCodec) -> Result<Self, Self::Error> {
        match codec {
            WaveSupportedCodec::Pcm => Ok(PcmCodecTag),
            _ => Err(SyphonError::Unsupported),
        }
    }
}

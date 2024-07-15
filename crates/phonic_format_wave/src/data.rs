use phonic_codec_pcm::{fill_pcm_spec, PcmCodecTag};
use phonic_core::PhonicError;
use phonic_io_core::{
    utils::FormatIdentifiers, CodecTag, DynFormat, DynFormatConstructor, FormatData, FormatTag,
    StdIoSource, StreamSpec,
};

use crate::WaveFormat;

pub static WAVE_IDENTIFIERS: FormatIdentifiers = FormatIdentifiers {
    file_extensions: &["wav", "wave"],
    mime_types: &["audio/vnd.wave", "audio/x-wav", "audio/wav", "audio/wave"],
    markers: &[b"RIFF", b"WAVE"],
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct WaveFormatTag;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WaveSupportedCodec {
    Pcm,
}

pub fn fill_wave_data<F>(data: &mut FormatData<F>) -> Result<(), PhonicError>
where
    F: FormatTag,
    WaveFormatTag: TryInto<F>,
{
    let expected_format = WaveFormatTag.try_into().ok();
    if data.format.is_some() && data.format != expected_format {
        return Err(PhonicError::InvalidData);
    } else {
        data.format = expected_format;
    }

    match data.streams.len() {
        0 => data.streams.push(StreamSpec::new()),
        1 => data.streams.first_mut().unwrap().fill()?,
        _ => return Err(PhonicError::Unsupported),
    }

    Ok(())
}

impl FormatTag for WaveFormatTag {
    type Codec = WaveSupportedCodec;

    fn fill_data(data: &mut FormatData<Self>) -> Result<(), PhonicError> {
        fill_wave_data(data)
    }
}

impl CodecTag for WaveSupportedCodec {
    fn fill_spec(spec: &mut StreamSpec<Self>) -> Result<(), PhonicError> {
        fill_pcm_spec(spec)
    }
}

impl From<PcmCodecTag> for WaveSupportedCodec {
    fn from(_: PcmCodecTag) -> Self {
        Self::Pcm
    }
}

impl TryFrom<WaveSupportedCodec> for PcmCodecTag {
    type Error = PhonicError;

    fn try_from(codec: WaveSupportedCodec) -> Result<Self, Self::Error> {
        match codec {
            WaveSupportedCodec::Pcm => Ok(PcmCodecTag),
            _ => Err(PhonicError::Unsupported),
        }
    }
}

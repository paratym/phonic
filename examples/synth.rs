use std::{
    fs::{create_dir_all, File},
    path::Path,
    time::Duration,
};
use syphon::{
    dsp::generators::SignalGenerator,
    io::{
        formats::{FormatIdentifier, WaveFormat},
        utils::copy,
        Format, FormatData, IntoFormatWriter, Stream, StreamSpec,
    },
    KnownSample, Sample, SampleType, Signal, SignalSpec, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let mut sine = SignalSpec::builder()
        .with_channels(1)
        .with_frame_rate(44100)
        .with_duration(Duration::from_secs(2))
        .generate_sine(440.0)?;

    let path = Path::new("./examples/generated/sine.wav");
    create_dir_all(path.parent().ok_or(SyphonError::IoError)?)?;

    let format = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| FormatIdentifier::FileExtension(ext))
        .and_then(|ref id| id.try_into().ok())
        .ok_or(SyphonError::Unsupported)?;

    let track_spec = StreamSpec::new()
        .with_sample_type(f32::TYPE)
        .with_decoded_spec((*sine.spec()).into());

    let format_data = FormatData::new()
        .with_format(format)
        .with_track(track_spec)
        .filled()?;

    let mut writer = File::create(path)?
        .into_format_writer(format_data)?
        .into_default_track()?
        .into_encoder()?
        .unwrap_f32_signal()?;

    let mut buf = [f32::ORIGIN; 1024];
    copy(&mut sine, &mut writer, &mut buf)
}

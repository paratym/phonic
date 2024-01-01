use std::{fs::File, time::Duration};
use syphon::{
    dsp::generators::SignalGenerator,
    io::{utils::copy, Format, FormatData, StreamSpec, StreamWriter, SyphonFormat, TryIntoFormatWriter},
    Sample, SampleType, Signal, SignalSpec, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let mut sine = SignalSpec::builder()
        .with_sample_type(SampleType::F32)
        .with_channels(1)
        .with_frame_rate(44100)
        .with_duration(Duration::from_secs(2))
        .generate_sine(440.0)?;
    
    let track_spec = StreamSpec::builder().with_decoded_spec(*sine.spec());
    let format_data = FormatData::builder()
        .with_format(SyphonFormat::Wave)
        .with_track(track_spec);

    let mut writer = File::create("./examples/generated/sine.wav")?
        .try_into_format_writer(format_data)?
        .into_default_track()?
        .into_encoder()?
        .unwrap_f32_signal()?;

    let mut buf = [f32::ORIGIN; 1024];
    copy(&mut sine, &mut writer, &mut buf)
}

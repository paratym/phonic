use std::{fs::File, time::Duration};
use syphon::{
    dsp::{adapters::SignalAdapter, generators::Sine},
    io::{utils::copy, Format, FormatData, StreamSpec, StreamWriter, SyphonFormat},
    Sample, Signal, SignalSpec, SyphonError, SampleType,
};

fn main() -> Result<(), SyphonError> {
    let signal_spec = SignalSpec::builder()
        .with_sample_type(SampleType::F32)
        .with_frame_rate(44100)
        .with_channels(1)
        .build()?;

    let mut sine = Sine::new(signal_spec, 440.0).adapt_duration(Duration::from_secs(2));

    let track_spec = StreamSpec::builder().with_decoded_spec(*sine.spec());
    let format_data = FormatData::builder()
        .with_format(SyphonFormat::Wave)
        .with_track(track_spec)
        .filled()?
        .build()?;

    let file = Box::new(File::create("./examples/generated/sine.wav")?);
    let mut signal_writer = format_data
        .construct_writer(file)?
        .into_default_track()?
        .into_encoder()?
        .unwrap_f32_signal()?;

    let mut buf = [f32::ORIGIN; 1024];
    copy(&mut sine, &mut signal_writer, &mut buf)
}

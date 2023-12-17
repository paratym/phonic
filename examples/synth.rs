use std::fs::File;
use syphon::{
    dsp::generators::Sine,
    io::{utils::copy, Format, FormatData, StreamSpec, StreamWriter, SyphonFormat},
    Sample, Signal, SignalSpec, SyphonError, SignalSpecBuilder,
};

fn main() -> Result<(), SyphonError> {
    let signal_spec = SignalSpec::<f32>::builder()
        .with_frame_rate(44100)
        .with_n_channels(1)
        .build()?;

    let mut sine = Sine::new(signal_spec, 440.0).adapt_seconds(2.0);
    
    let track_spec = StreamSpec::builder().with_decoded_spec(*sine.spec());
    let format_data = FormatData::builder()
        .with_format(SyphonFormat::Wave)
        .with_track(track_spec)
        .filled()?
        .build()?;

    let file = Box::new(File::create("./examples/generated/sine.wav")?);
    let mut signal_writer = format_data
        .writer(file)?
        .into_default_track()?
        .into_encoder()?
        .unwrap_f32_signal()?;

    let mut buf = [f32::ORIGIN; 1024];
    copy(&mut sine, &mut signal_writer, &mut buf)
}

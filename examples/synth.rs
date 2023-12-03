use std::{fs::File, time::Duration};
use syphon::{
    dsp::generators::Sine,
    io::{utils::copy, Format, FormatData, StreamSpec, StreamWriter, SyphonFormat},
    Sample, Signal, SignalSpec, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let spec = SignalSpec::<f32>::builder().stereo().hz(44100).build()?;
    let mut sine = Sine::new(spec, 440.0).adapt_seconds(2.0);

    let file = Box::new(File::create("./examples/samples/sine.wav")?);
    let format_writer = FormatData::builder()
        .format(SyphonFormat::Wave)
        .track(StreamSpec::builder().decoded_spec((*sine.spec()).into()))
        .build()?
        .writer(file)?;

    let mut encoder = format_writer
        .into_track(0)?
        .into_encoder()?
        .unwrap_f32_signal()?;

    let mut buf = [f32::ORIGIN; 1024];
    copy(&mut sine, &mut encoder, &mut buf)
}

use std::fs::File;
use syphon::{
    dsp::generators::Sine,
    io::{utils::copy, Format, FormatData, StreamSpec, StreamWriter, SyphonFormat},
    Sample, Signal, SignalSpec, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let spec = SignalSpec::builder()
        .sample::<f32>()
        // .stereo()
        .hz(44100)
        .n_blocks(88200)
        .build()?;

    let mut sine = Sine::new(spec, 440.0);

    let file = Box::new(File::create("./examples/samples/sine.wav")?);
    let format_writer = FormatData::builder()
        .format(SyphonFormat::Wave)
        .track(StreamSpec::builder().decoded_spec((*sine.spec()).into()))
        .build()?
        .writer(file)?;

    let mut encoder = format_writer
        .into_track_stream(0)?
        .into_encoder()?
        .unwrap_f32_signal()?;

    let mut buf = [f32::MID; 1024];
    copy(&mut sine, &mut encoder, &mut buf)
}

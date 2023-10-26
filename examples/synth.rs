use std::{fs::File, path::Path};
use syphon::{
    dsp::generators::Sine,
    io::{Format, FormatData, StreamSpec, StreamWriter, SyphonFormat, utils::copy},
    Sample, Signal, SignalSpec, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let dst_path = Path::new("./examples/samples/synth.wav");
    let dst_file = Box::new(File::create(dst_path)?);

    let spec = SignalSpec::builder()
        .sample_type::<f32>()
        .sample_rate(44100)
        .n_channels(1)
        // .block_size(1) // default
        .n_blocks(44100)
        .build()?;

    let mut sine = Sine::new(spec, 440.0);

    let mut encoder = FormatData::new()
        .format(SyphonFormat::Wave)
        .track(StreamSpec::builder().decoded_spec((*sine.spec()).into()))
        .fill()?
        .write(dst_file)?
        .into_default_track()?
        .encoder()?
        .adapt_sample_type::<f32>();

    let mut buf = [f32::MID; 1024];
    copy(&mut sine, &mut encoder, &mut buf)
}

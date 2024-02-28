use std::{
    fs::{create_dir_all, File},
    io::copy,
    path::Path,
    time::Duration,
};
use syphon::{
    dsp::generators::SineGenerator,
    io::{codecs::PcmCodec, formats::WaveFormat, Format, FormatData, FormatWriter, Stream},
    signal::SignalSpec,
    SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let spec = SignalSpec::builder()
        .with_channels(1)
        .with_frame_rate(44100)
        .with_duration(Duration::from_secs(2))
        .build()?;

    let mut sine = SineGenerator::new(spec, 440.0);
    let mut encoder = PcmCodec::from_signal(&mut sine)?;

    let path = Path::new("./examples/generated/sine.wav");
    create_dir_all(path.parent().ok_or(SyphonError::IoError)?)?;
    let mut file = File::create(path)?;

    let mut muxer = WaveFormat::new(&mut file)?;
    let data = <FormatData>::new().with_stream(*encoder.spec());
    muxer.write_data(&data)?;

    copy(&mut encoder, &mut muxer.as_default_stream()?)?;
    Ok(())
}

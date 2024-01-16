use std::{
    fs::{create_dir_all, File},
    io::copy,
    path::Path,
    time::Duration,
};
use syphon::{
    dsp::generators::SineGenerator,
    io::{codecs::PcmCodec, formats::WaveFormat, FormatData, Stream, SyphonFormat},
    signal::SignalSpec,
    SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let spec = SignalSpec::builder()
        .with_channels(1)
        .with_frame_rate(44100)
        .with_duration(Duration::from_secs(2))
        .build()?;

    let sine = SineGenerator::new(spec, 440.0);
    let mut encoder = PcmCodec::from_signal(sine)?;

    let path = Path::new("./examples/generated/sine.wav");
    create_dir_all(path.parent().ok_or(SyphonError::IoError)?)?;
    let file = File::create(path)?;

    let data = FormatData::<SyphonFormat>::new().with_track(None, *encoder.spec());
    let mut formatter = WaveFormat::write(file, data.try_into()?)?;

    copy(&mut encoder, &mut formatter)?;
    Ok(())
}

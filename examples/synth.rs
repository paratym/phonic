use std::{
    fs::{create_dir_all, File},
    io::copy,
    path::Path,
    time::Duration,
};
use syphon::{
    dsp::generators::SineGenerator,
    io::{codecs::PcmCodec, formats::WaveFormat, FormatData, Stream},
    signal::SignalSpec,
    SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let signal_spec = SignalSpec::builder()
        .with_channels(1)
        .with_frame_rate(44100)
        .with_duration(Duration::from_secs(2))
        .build()?;

    let sine = SineGenerator::new(signal_spec, 440.0);
    let mut encoder = PcmCodec::from_signal(sine)?;

    let path = Path::new("./examples/generated/sine.wav");
    create_dir_all(path.parent().ok_or(SyphonError::IoError)?)?;
    let file = File::create(path)?;

    let data = FormatData::new().with_track(*Stream::spec(&encoder));
    let mut formatter = WaveFormat::write(file, data.try_into()?)?;

    copy(&mut encoder, &mut formatter)?;
    Ok(())
}

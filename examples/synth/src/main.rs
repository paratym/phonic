use std::{
    fs::{create_dir_all, File},
    path::Path,
    time::Duration,
};
use syphon::{
    io::{
        codecs::pcm::PcmCodec, formats::wave::WaveFormat, Format, FormatData, FormatWriter, Stream,
        StreamWriter,
    },
    signal::SignalSpec,
    synth::generators::SineGenerator,
    SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let path = Path::new("./examples/generated/sine.wav");
    create_dir_all(path.parent().ok_or(SyphonError::IoError)?)?;
    let mut file = File::create(path)?;

    let spec = SignalSpec::builder()
        .with_channels(1)
        .with_frame_rate(44100)
        .with_duration(Duration::from_secs(2))
        .build()?;

    let mut sine = SineGenerator::new(spec, 440.0);
    let mut encoder = PcmCodec::from_signal(&mut sine)?;

    let data = FormatData::new().with_stream(*encoder.spec());
    let mut muxer = <WaveFormat<_>>::new(&mut file)?;

    muxer.write_data(&data)?;
    muxer.as_default_stream()?.write_all(&mut encoder)?;

    Ok(())
}

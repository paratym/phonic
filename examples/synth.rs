use std::{
    fs::{create_dir_all, File},
    io::copy,
    path::Path,
    time::Duration,
};
use syphon::{
    dsp::generators::SignalGenerator,
    io::{codecs::PcmCodec, formats::WaveFormat, FormatData, Stream, StreamSpec},
    Signal, SignalSpec, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let sine = SignalSpec::builder()
        .with_channels(1)
        .with_frame_rate(44100)
        .with_duration(Duration::from_secs(2))
        .generate_sine(440.0)?;

    let spec = StreamSpec::builder()
        .with_sample_type::<f32>()
        .with_decoded_spec(sine.spec().clone().into());
    
    let mut encoder = PcmCodec::new(sine, spec)?;

    let path = Path::new("./examples/generated/sine.wav");
    create_dir_all(path.parent().ok_or(SyphonError::IoError)?)?;
    let file = File::create(path)?;

    let data = FormatData::new().with_track(Stream::spec(&encoder).clone().into());
    let mut formatter = WaveFormat::write(file, data.try_into()?)?;

    copy(&mut encoder, &mut formatter)?;
    Ok(())
}

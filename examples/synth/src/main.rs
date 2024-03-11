use std::{
    fs::{create_dir_all, remove_file, File},
    path::Path,
    time::Duration,
};
use phonic::{
    io::{
        codecs::pcm::PcmCodec, formats::wave::WaveFormat, Format, FormatData, FormatWriter, Stream,
        StreamWriter,
    },
    signal::SignalSpec,
    synth::generators::SineGenerator,
    PhonicError,
};

fn main() -> Result<(), PhonicError> {
    let spec = SignalSpec::builder()
        .with_channels(1)
        .with_frame_rate(44100)
        .with_duration(Duration::from_secs(2))
        .build()?;

    let mut sine = SineGenerator::new(spec, 440.0);
    let mut encoder = PcmCodec::from_signal(&mut sine)?;

    let path = Path::new("sine.wav");
    let mut file = File::create(path)?;

    let data = FormatData::new().with_stream(*encoder.spec());
    let result = <WaveFormat<_>>::new(&mut file)
        .and_then(|mut wave| wave.write_data(&data).map(|_| wave))
        .and_then(|mut wave| wave.into_default_stream())
        .and_then(|mut stream| stream.copy_all(&mut encoder));

    match result {
        Err(e) => {
            if let Err(e) = remove_file(path) {
                println!("error removing malformed output file: {e}");
            }

            Err(e)
        }
        Ok(n) => {
            println!("wrote {n} bytes to {}", path.display());
            Ok(())
        }
    }
}

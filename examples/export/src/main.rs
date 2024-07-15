use phonic::{
    io::{
        codecs::pcm::PcmCodec, formats::wave::WaveFormat, FormatData, FormatWriter, Stream,
        StreamWriter,
    },
    signal::{utils::SignalGenerator, SignalReader, SignalSpec},
    PhonicError,
};
use std::{
    f64::consts::PI,
    fs::{remove_file, File},
    path::Path,
    time::Duration,
};

fn main() -> Result<(), PhonicError> {
    let spec = SignalSpec::builder()
        .with_channels(1)
        .with_frame_rate(44100)
        .with_duration(Duration::from_secs(2))
        .build()?;

    let mut sine = SignalGenerator::new(spec, |signal_i: u64, buf: &mut [f32]| {
        let mut ts = signal_i as f64 / spec.sample_rate() as f64;
        let t_step = 1.0 / spec.frame_rate as f64;

        let n_channels = spec.channels.count() as usize;
        let frames = buf.chunks_exact_mut(n_channels);
        let n_samples = frames.len() * n_channels;

        for frame in frames {
            let mut sine_sample = (ts * 440.0 * 2.0 * PI) as f32;
            if sine_sample > 0.0 {
                sine_sample = 1.0 - sine_sample
            } else {
                sine_sample = -1.0 - sine_sample;
            }

            frame.fill(sine_sample);
            ts += t_step;
        }

        Ok(n_samples)
    });

    let path = Path::new("sine.wav");
    let mut file = File::create(path)?;

    let result = export(&mut sine, &mut file);
    if let Err(_) = result {
        if let Err(e) = remove_file(path) {
            println!("error removing malformed output file: {e}");
        }
    }

    result
}

fn export(
    signal: &mut impl SignalReader<Sample = f32>,
    writer: &mut impl std::io::Write,
) -> Result<(), PhonicError> {
    let mut formatter = <WaveFormat<_>>::new(writer)?;
    let mut encoder = PcmCodec::from_signal(signal)?;
    let data = FormatData::new().with_stream(*encoder.spec());

    FormatWriter::write(&mut formatter, (&data).into())?;
    StreamWriter::copy_all(&mut formatter, &mut encoder)?;

    Ok(())
}

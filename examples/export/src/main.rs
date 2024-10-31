use phonic::{
    io::{
        codecs::pcm::PcmCodec, formats::wave::WaveFormat, CodecConstructor, FormatConstructor,
        FormatWriter, Stream, StreamWriter,
    },
    signal::{adapters::SignalAdapter, utils::SignalGenerator, SignalReader, SignalSpec},
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
        .with_sample_rate(44100)
        .build()?;

    let mut sine = SignalGenerator::new(spec, |pos: u64, buf: &mut [f32]| {
        let n_channels = spec.channels.count() as usize;
        let frames = buf.chunks_exact_mut(n_channels);
        let n_samples = frames.len() * n_channels;

        let mut ts = pos as f64 / spec.sample_rate as f64;
        let t_step = 1.0 / spec.sample_rate as f64;

        for frame in frames {
            let sine_sample = (ts * 440.0 * 2.0 * PI).sin() as f32;
            frame.fill(sine_sample);
            ts += t_step;
        }

        Ok(n_samples)
    })
    .adapt_len_duration(Duration::from_secs(1));

    let path = Path::new("sine.wav");
    let mut file = File::create(path)?;

    match export(&mut sine, &mut file) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("error exporting signal: {e}");
            remove_file(path).map_err(Into::into)
        }
    }
}

fn export(
    signal: &mut impl SignalReader<Sample = f32>,
    writer: &mut impl std::io::Write,
) -> Result<(), PhonicError> {
    let mut codec = PcmCodec::encoder(signal)?;
    let mut format = <WaveFormat<_>>::write_index(writer, [*codec.stream_spec()])?;

    format.copy_all(&mut codec, true)?;
    format.finalize()
}

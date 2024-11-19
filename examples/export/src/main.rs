use phonic::{
    dsp::{gen::Osc, utils::UtilSignalExt},
    io::{
        codecs::pcm::PcmCodec, formats::wave::WaveFormat, CodecConstructor, Format,
        FormatConstructor, FormatWriter, Stream, StreamWriter,
    },
    signal::{SignalReader, SignalSpec},
    PhonicError,
};
use std::{
    fs::{remove_file, File},
    path::Path,
};

fn main() -> Result<(), PhonicError> {
    let spec = SignalSpec::new(48000, 1);
    let mut sine = Osc::sin(spec, 440.0, 0.6, 0.0).slice(0, 48000);

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

    format.as_default_stream()?.copy_all(&mut codec, true)?;
    format.finalize()
}

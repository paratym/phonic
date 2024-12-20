use phonic::{
    dsp::{gen::Osc, utils::UtilSignalExt},
    io::{
        codecs::pcm::PcmCodec, formats::wave::WaveFormat, CodecConstructor, Format,
        FormatConstructor, FormatWriter, Stream,
    },
    PhonicResult, SignalReader, SignalSpec,
};
use std::{
    fs::{remove_file, File},
    path::Path,
    time::Duration,
};

fn main() -> PhonicResult<()> {
    let spec = SignalSpec::new(48000, 1);
    let mut sine = Osc::sin(spec, 440.0, 0.6, 0.0).slice_from_start(Duration::from_secs(1));

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
) -> PhonicResult<()> {
    // let mut codec = PcmCodec::encoder(signal)?;
    // let mut format = <WaveFormat<_>>::write_index(writer, [*codec.stream_spec()])?;
    //
    // format.as_primary_stream()?.copy_all_poll(&mut codec)?;
    // format.finalize()
    todo!()
}

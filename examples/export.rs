use phonic::{
    dsp::{gen::Osc, utils::DspUtilsExt},
    io::{
        pcm::PcmCodec,
        utils::{FormatUtilsExt, StreamUtilsExt},
        wave::WaveFormat,
        CodecFromSignal, FormatConstructor, Stream,
    },
    PhonicResult, SignalReader, SignalSpec,
};
use std::{
    fs::{remove_file, File},
    mem::MaybeUninit,
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
    let mut codec = PcmCodec::default_from_signal(signal)?.polled();
    let mut format = <WaveFormat<_>>::write_index(writer, [*codec.stream_spec()])?;
    let mut buf = [MaybeUninit::<u8>::uninit(); 4096];

    (&mut format)
        .into_primary_stream()?
        .polled()
        .copy_all(&mut codec, &mut buf)?;

    // TODO: format.finalize()
    Ok(())
}

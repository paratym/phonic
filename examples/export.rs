use phonic::{
    dsp::utils::{DspUtilsExt, Osc},
    io::{
        codecs::pcm::PcmCodec,
        formats::wave::WaveFormat,
        utils::{FormatUtilsExt, StreamUtilsExt},
        CodecFromSignal, FormatFromWriter, Stream,
    },
    PhonicResult, SignalReader, SignalSpec,
};
use std::{
    fs::{remove_file, File},
    io::{Seek, Write},
    mem::MaybeUninit,
    path::Path,
    time::Duration,
};

fn main() -> PhonicResult<()> {
    let spec = SignalSpec::stereo(48000);
    let sine = Osc::hz(440.0)
        .sin(spec)
        .slice_from_start(Duration::from_secs(1));

    let path = Path::new("sine.wav");
    let mut file = File::create(path)?;

    match export(sine, &mut file) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("error exporting signal: {e}");
            remove_file(path).map_err(Into::into)
        }
    }
}

fn export<R, W>(signal: R, writer: W) -> PhonicResult<()>
where
    R: SignalReader<Sample = f32>,
    W: Write + Seek,
{
    let codec = PcmCodec::default_from_signal(signal)?.polled();
    let format = <WaveFormat<_>>::write_index(writer, [*codec.stream_spec()])?;
    let mut buf = [MaybeUninit::<u8>::uninit(); 4096];

    format
        .finalize_on_drop()
        .into_primary_stream()?
        .polled()
        .copy_all(codec, &mut buf)
}

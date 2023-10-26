use std::{fs::File, path::Path};
use syphon::{
    dsp::{adapters::NBlocksAdapter, generators::SineGenerator},
    io::{
        codecs::pcm::{fill_pcm_spec, PcmCodec},
        formats::Wave,
        utils::pipe,
        EncodedStreamSpec, FormatData, SyphonCodec,
    },
    Sample, Signal, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let sine = SineGenerator::new(440.0, 44100);
    let mut sine_one_sec = NBlocksAdapter::from_signal(sine, 44100);

    let dst_path = Path::new("./examples/samples/synth.wav");
    let dst_file = File::create(dst_path)?;

    let mut format_data = FormatData::new().track(
        EncodedStreamSpec::builder()
            .codec(SyphonCodec::Pcm)
            .decoded_spec(sine_one_sec.spec().clone().into()),
    );

    fill_pcm_spec(&mut format_data.tracks[0])?;

    let stream_writer = Wave::write(dst_file, format_data.try_into()?)?.into_stream()?;
    let mut encoder = PcmCodec::from_stream(stream_writer)?;

    let mut buf = [f32::MID; 1024];
    pipe(&mut sine_one_sec, &mut encoder, &mut buf)
}

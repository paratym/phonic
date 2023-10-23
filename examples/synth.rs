use std::{fs::File, path::Path};
use syphon::{
    dsp::{adapters::NFramesAdapter, generators::SineGenerator},
    io::{
        codecs::PcmCodec, formats::WavFormat, utils::pipe_buffered, EncodedStreamSpec, FormatData,
        SyphonCodec,
    },
    Sample, SampleStream, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let sine = SineGenerator::new(440.0, 44100);
    let mut sample_reader = NFramesAdapter::from_stream(sine, 44100);

    let dst_path = Path::new("./examples/samples/synth.wav");
    let dst_file = File::create(dst_path)?;

    let mut encoded_spec = EncodedStreamSpec::builder()
        .codec(SyphonCodec::Pcm)
        .decoded_spec(sample_reader.spec().clone().into());

    PcmCodec::<()>::fill_spec(&mut encoded_spec)?;

    let format_data = FormatData::new().track(encoded_spec);
    let stream_writer = WavFormat::write(dst_file, format_data.try_into()?)?.into_dyn_stream()?;
    let mut encoder = PcmCodec::from_stream(stream_writer)?;

    let mut buf = [f32::MID; 1024];
    pipe_buffered(&mut sample_reader, &mut encoder, &mut buf)
}

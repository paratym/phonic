use std::fs::File;

use syphon::{
    core::SyphonError,
    io::{formats::WavReader, syphon_decoder_registry, AudioFormatReader},
};

fn main() -> Result<(), SyphonError> {
    let file = File::open("").unwrap();
    let mut wav_reader = WavReader::new(Box::new(file));
    let (_codec_id, spec_builder) = wav_reader.read_spec()?;
    let decoder = syphon_decoder_registry().construct_decoder(
        "pcm",
        Box::new(wav_reader),
        spec_builder.try_build()?,
    );

    Ok(())
}

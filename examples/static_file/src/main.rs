use std::fs::File;

use syphon::{
    core::SyphonError,
    io::{syphon_decoder_registry, syphon_format_reader_registry, SyphonFormatKey},
};

fn main() -> Result<(), SyphonError> {
    let file = File::open("").unwrap();
    let mut format_reader = syphon_format_reader_registry()
        .construct_reader(&SyphonFormatKey::Wav, Box::new(file))
        .unwrap();

    let format_data = format_reader.read_spec()?;
    let decoder = syphon_decoder_registry().construct_decoder(
        &format_data.codec_key.unwrap(),
        Box::new(format_reader),
        format_data.spec.try_build()?,
    );

    Ok(())
}

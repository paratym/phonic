use std::fs::File;

use syphon::{
    core::SyphonError,
    io::{syphon_codec_registry, syphon_format_registry, TrackReader},
};

fn main() -> Result<(), SyphonError> {
    let file = File::open("").unwrap();

    let mut format_reader = syphon_format_registry()
        .resolve_reader(Box::new(file), None)
        .ok_or(SyphonError::Unsupported)?;

    format_reader.read_headers()?;
    let track_reader = TrackReader::default(format_reader);

    // let decoder = syphon_codec_registry().construct_decoder(
    //     &format_data.codec_key.unwrap(),
    //     Box::new(format_reader),
    //     format_data.spec.try_build()?,
    // );

    Ok(())
}

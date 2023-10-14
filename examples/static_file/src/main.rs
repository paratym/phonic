use std::fs::File;
use syphon::{
    io::{syphon_codec_registry, syphon_format_registry, TrackReader},
    SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let file = File::open("")?;
    let mut format_reader = syphon_format_registry()
        .resolve_reader(file, None)
        .ok_or(SyphonError::Unsupported)?;

    format_reader.read_headers()?;
    let track_reader = TrackReader::default(format_reader.as_mut())?;

    // let decoder = syphon_codec_registry().construct_decoder(
    //     &format_data.codec_key.unwrap(),
    //     Box::new(format_reader),
    //     format_data.spec.try_build()?,
    // );

    Ok(())
}

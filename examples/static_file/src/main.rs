use std::fs::File;
use syphon::{
    io::{syphon_codec_registry, syphon_format_registry, TrackReader},
    SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let file = File::open("")?;
    let mut format_reader = syphon_format_registry().resolve_reader(file, None)?;

    format_reader.read_headers()?;
    let track_reader = TrackReader::default(format_reader)?;
    let decoder = syphon_codec_registry().construct_decoder(track_reader)?;

    Ok(())
}

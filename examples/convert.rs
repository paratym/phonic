use std::{
    fs::{create_dir_all, File},
    path::Path,
};
use syphon::{
    io::{
        formats::SyphonFormat,
        formats::{FormatIdentifier, FormatTag},
        Format, FormatData, StreamReader, StreamSpec, StreamWriter,
    },
    signal::{utils::copy, Sample},
    SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/generated/sine.wav");
    let src_file = File::open(src_path)?;
    let src_fmt = SyphonFormat::try_from(&FormatIdentifier::try_from(src_path)?)?;

    let mut decoder = src_fmt
        .construct_reader(src_file)?
        .into_default_stream()?
        .into_decoder()?
        .adapt_sample_type();

    let dst_path = Path::new("./examples/generated/sine_converted.wav");
    create_dir_all(dst_path.parent().ok_or(SyphonError::IoError)?)?;
    let dst_file = File::create(dst_path)?;

    let dst_data = FormatData::new().with_stream((&decoder).into());
    let dst_fmt = SyphonFormat::try_from(&FormatIdentifier::try_from(dst_path)?)?;

    let mut encoder = dst_fmt
        .construct_writer(dst_file, dst_data)?
        .into_default_stream()?
        .into_encoder()?
        .unwrap_i16_signal()?;

    let mut buf = [i16::ORIGIN; 2048];
    copy(&mut decoder, &mut encoder, &mut buf)
}

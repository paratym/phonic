use std::{
    fs::{create_dir_all, File},
    path::Path,
};
use syphon::{
    io::{
        formats::FormatIdentifier, utils::copy, Format, FormatData, IntoFormatReader,
        IntoFormatWriter, Stream, StreamSpec,
    },
    Sample, SampleType, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/generated/sine.wav");
    let src_fmt_id = src_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| FormatIdentifier::FileExtension(ext));

    let mut decoder = File::open(src_path)?
        .resolve_format_reader(src_fmt_id.as_ref())?
        .into_default_track()?
        .into_decoder()?
        .into_adapter();

    let dst_path = Path::new("./examples/generated/sine_converted.wav");
    create_dir_all(dst_path.parent().ok_or(SyphonError::IoError)?)?;

    let dst_fmt = dst_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| FormatIdentifier::FileExtension(ext))
        .and_then(|ref id| id.try_into().ok())
        .ok_or(SyphonError::Unsupported)?;

    let dst_stream_spec = StreamSpec::new()
        .with_sample_type(SampleType::I16)
        .with_decoded_spec((*decoder.spec()).into());

    let dst_fmt_data = FormatData::new()
        .with_format(dst_fmt)
        .with_track(dst_stream_spec)
        .filled()?;

    let mut encoder = File::create(dst_path)?
        .into_format_writer(dst_fmt_data)?
        .into_default_track()?
        .into_encoder()?
        .unwrap_i16_signal()?;

    let mut buf = [i16::ORIGIN; 1024];
    copy(&mut decoder, &mut encoder, &mut buf)
}

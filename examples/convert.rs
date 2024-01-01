use std::{fs::File, path::Path};
use syphon::{
    io::{
        formats::FormatIdentifier, utils::copy, Format, FormatData, StreamReader, StreamSpec,
        StreamWriter, TryIntoFormatReader, TryIntoFormatWriter,
    },
    Sample, SampleType, Signal, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/generated/sine.wav");
    let src_fmt_id = src_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| FormatIdentifier::FileExtension(ext))
        .unwrap_or_default();

    let mut decoder = File::open(src_path)?
        .try_into_format_reader(&src_fmt_id)?
        .into_default_track()?
        .into_decoder()?
        .into_adapter();

    let dst_path = Path::new("./examples/generated/sine_converted.wav");
    let dst_fmt_id = dst_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| FormatIdentifier::FileExtension(ext))
        .unwrap_or_default();

    let dst_signal_spec = decoder
        .spec()
        .into_builder()
        .with_sample_type(SampleType::I16);

    let dst_stream_spec = StreamSpec::builder().with_decoded_spec(dst_signal_spec);
    let dst_fmt_data = FormatData::builder()
        .with_format(&dst_fmt_id)
        .with_track(dst_stream_spec);

    let mut encoder = File::create(dst_path)?
        .try_into_format_writer(dst_fmt_data)?
        .into_default_track()?
        .into_encoder()?
        .unwrap_i16_signal()?;

    let mut buf = [i16::ORIGIN; 1024];
    copy(&mut decoder, &mut encoder, &mut buf)
}

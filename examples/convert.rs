use std::{fs::File, path::Path};
use syphon::{
    io::{
        formats::FormatIdentifier, utils::copy, Format, FormatData, StreamReader, StreamSpec,
        StreamWriter, SyphonFormat,
    },
    Sample, SampleType, Signal, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/generated/sine.wav");
    let mut src_file = Box::new(File::open(src_path)?);
    let src_fmt_id = src_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| FormatIdentifier::FileExtension(ext));

    let mut decoder = SyphonFormat::resolve(&mut src_file, src_fmt_id)?
        .construct_reader(src_file)?
        .into_default_track()?
        .into_decoder()?
        .adapt_sample_type();

    let dst_signal_spec = decoder
        .spec()
        .into_builder()
        .with_sample_type(SampleType::I16);
    
    let dst_stream_spec = StreamSpec::builder().with_decoded_spec(dst_signal_spec);
    let dst_fmt_data = FormatData::builder()
        .with_format(SyphonFormat::Wave)
        .with_track(dst_stream_spec)
        .filled()?
        .build()?;

    let dst_path = Path::new("./examples/generated/sine_converted.wav");
    let dst_file = Box::new(File::create(dst_path)?);

    let mut encoder = dst_fmt_data
        .construct_writer(dst_file)?
        .into_default_track()?
        .into_encoder()?
        .unwrap_i16_signal()?;

    let mut buf = [i16::ORIGIN; 1024];
    copy(&mut decoder, &mut encoder, &mut buf)
}

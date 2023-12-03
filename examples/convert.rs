use std::{fs::File, path::Path};
use syphon::{
    io::{
        formats::FormatIdentifier, utils::copy, Format, FormatData, StreamReader, StreamSpec,
        StreamWriter, SyphonFormat,
    },
    Sample, Signal, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/samples/sine.wav");
    let src_file = Box::new(File::open(src_path)?);
    let format_identifier = src_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| FormatIdentifier::FileExtension(ext));

    let mut decoder = SyphonFormat::resolve_reader(src_file, format_identifier)?
        .into_default_track()?
        .into_decoder()?
        .adapt_sample_type::<f32>();

    let data = FormatData::builder()
        .format(SyphonFormat::Wave)
        .track(StreamSpec::builder().decoded_spec(decoder.spec().clone().into()))
        .build()?;

    let dst_file = Box::new(File::create("./examples/samples/sine_converted.wav")?);
    let mut encoder = data
        .writer(dst_file)?
        .into_default_track()?
        .into_encoder()?
        .unwrap_f32_signal()?;

    let mut buf = [f32::ORIGIN; 1024];
    copy(&mut decoder, &mut encoder, &mut buf)
}

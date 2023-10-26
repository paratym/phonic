use std::{fs::File, io::Read, path::Path};
use syphon::{
    io::{
        formats::FormatIdentifier, utils::copy, Format, FormatData, SignalWriterRef, StreamReader,
        StreamSpec, StreamSpecBuilder, SyphonCodec, SyphonFormat, StreamWriter,
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
        .decoder()?
        .adapt_sample_type::<f32>();

    let dst_file = Box::new(File::create("./examples/samples/sine_converted.wav")?);
    let mut encoder = FormatData::new()
        .format(SyphonFormat::Wave)
        .track(StreamSpec::builder().decoded_spec(decoder.spec().clone().into()))
        .write(dst_file)?
        .into_default_track()?
        .encoder()?
        .adapt_sample_type::<f32>();

    let mut buf = [f32::MID; 1024];
    copy(&mut decoder, &mut encoder, &mut buf)
}

use std::{
    fs::{create_dir_all, File},
    path::Path,
};
use syphon::{
    io::{
        formats::{FormatIdentifier, FormatTag, SyphonFormat},
        Format, FormatData, FormatReader, StreamReader, StreamSpec, StreamWriter,
    },
    signal::{utils::copy, Sample},
    SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/generated/sine.wav");
    let src_file = File::open(src_path)?;

    let src_fmt = SyphonFormat::try_from(&FormatIdentifier::try_from(src_path)?)?;
    let mut demuxer = src_fmt.demux_reader(src_file)?;
    demuxer.read_data()?;

    let mut decoder = demuxer
        .into_default_stream()?
        .into_decoder()?
        .adapt_sample_type();

    let dst_path = Path::new("./examples/generated/sine_converted.wav");
    create_dir_all(dst_path.parent().ok_or(SyphonError::IoError)?)?;
    let dst_file = File::create(dst_path)?;

    let dst_fmt = SyphonFormat::try_from(&FormatIdentifier::try_from(dst_path)?)?;
    let mut muxer = dst_fmt.mux_writer(dst_file)?;

    let dst_stream_spec = StreamSpec::from(&decoder);
    let dst_data = FormatData::new()
        .with_format(dst_fmt)
        .with_stream(dst_stream_spec)
        .filled()?;

    muxer.write_data(&dst_data)?;

    let mut encoder = muxer
        .into_default_stream()?
        .into_encoder()?
        .unwrap_i16_signal()?;

    let mut buf = [i16::ORIGIN; 2048];
    copy(&mut decoder, &mut encoder, &mut buf)?;
    Ok(())
}

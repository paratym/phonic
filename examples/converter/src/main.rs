use std::{
    fs::{create_dir_all, File},
    path::Path,
};
use syphon::{
    io::{
        utils::FormatIdentifier, DynFormatConstructor, DynStream, Format, FormatData, FormatReader,
        KnownFormat, StreamReader, StreamSpec, StreamWriter,
    },
    signal::SignalWriter,
    SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/generated/sine.wav");
    let src_file = File::open(src_path)?;

    let src_fmt = KnownFormat::try_from(&FormatIdentifier::try_from(src_path)?)?;
    let mut demuxer = src_fmt.from_std_io(src_file)?;
    demuxer.read_data()?;

    let mut decoder = demuxer
        .into_default_stream()?
        .into_codec()?
        .adapt_sample_type();

    let dst_path = Path::new("./examples/generated/sine_converted.wav");
    create_dir_all(dst_path.parent().ok_or(SyphonError::IoError)?)?;
    let dst_file = File::create(dst_path)?;

    let dst_fmt = KnownFormat::try_from(&FormatIdentifier::try_from(dst_path)?)?;
    let mut muxer = dst_fmt.from_std_io(dst_file)?;

    let dst_stream_spec = StreamSpec::from(&decoder);
    let dst_data = FormatData::new()
        .with_format(dst_fmt)
        .with_stream(dst_stream_spec)
        .filled()?;

    muxer.write_data(&dst_data)?;
    muxer
        .into_default_stream()?
        .into_codec()?
        .unwrap_i16_signal()?
        .copy_all(&mut decoder)?;

    Ok(())
}

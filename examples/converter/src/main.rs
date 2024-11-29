use phonic::{
    dsp::ops::TaggedSignalExt,
    io::{
        utils::FormatIdentifier, DynFormatConstructor, DynStream, Format, KnownFormat, StreamSpec,
    },
    signal::{utils::PollSignalWriter, PhonicError, PhonicResult},
};
use std::{
    fs::{create_dir_all, File},
    path::Path,
};

fn main() -> PhonicResult<()> {
    let src_path = Path::new("./examples/generated/sine.wav");
    let src_file = File::open(src_path)?;

    let src_fmt = KnownFormat::try_from(FormatIdentifier::try_from(src_path)?)?;
    let mut decoder = src_fmt
        .read_index(src_file)?
        .into_default_stream()?
        .into_decoder()?
        .convert();

    let spec = StreamSpec::builder()
        .with_decoded_spec(*decoder.spec())
        .with_sample_type::<i16>()
        .inferred()?
        .build()?;

    let dst_path = Path::new("./examples/generated/sine_converted.wav");
    create_dir_all(dst_path.parent().ok_or(PhonicError::NotFound)?)?;
    let dst_file = File::create(dst_path)?;

    let dst_fmt = KnownFormat::try_from(FormatIdentifier::try_from(dst_path)?)?;
    let muxer = dst_fmt.write_index(dst_file, [spec])?;
    let mut encoder = muxer.into_default_stream()?.into_decoder()?.unwrap_i16()?;

    encoder.copy_all(&mut decoder)?;
    // TODO: muxer.finalize()

    Ok(())
}

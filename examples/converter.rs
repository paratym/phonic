use phonic::{
    dsp::ops::TaggedSignalExt,
    io::{
        dynamic::{DynFormatConstructor, DynStream, FormatIdentifier},
        utils::FormatUtilsExt,
        StreamSpec,
    },
    utils::{DefaultSizedBuf, SignalUtilsExt, SizedBuf},
    PhonicError, PhonicResult,
};
use std::{
    fs::{create_dir_all, File},
    path::Path,
};

fn main() -> PhonicResult<()> {
    let src_path = Path::new("sine.wav");
    let src_file = File::open(src_path)?;

    let src_fmt = FormatIdentifier::try_from(src_path)?
        .known_format()
        .ok_or(PhonicError::Unsupported)?
        .read_index(src_file)?
        .finalize_on_drop();

    let decoder = src_fmt.into_primary_stream()?.into_decoder()?;
    let spec = StreamSpec::builder()
        .with_decoded_spec(*decoder.spec())
        .with_sample_type::<i16>()
        .inferred()?;

    let dst_path = Path::new("sine_i16.wav");
    create_dir_all(dst_path.parent().ok_or(PhonicError::NotFound)?)?;
    let dst_file = File::create(dst_path)?;

    let dst_fmt = FormatIdentifier::try_from(dst_path)?
        .known_format()
        .ok_or(PhonicError::Unsupported)?
        .write_index(dst_file, [spec])?
        .finalize_on_drop();

    let encoder = dst_fmt
        .into_primary_stream()?
        .into_decoder()?
        .unwrap_i16()
        .unwrap();

    let mut buf = DefaultSizedBuf::<i16>::uninit();
    encoder.copy_all(decoder.convert(), &mut buf)
}

use std::{fs::File, path::Path};
use syphon::{
    io::{codecs::PcmCodec, formats::WavFormat, utils::pipe_buffered},
    Sample, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/samples/sine.wav");
    let src_file = File::open(src_path)?;

    let format_reader = WavFormat::read(src_file)?;
    let wav_header = *format_reader.header();

    let stream_reader = format_reader.into_dyn_stream()?;
    let mut decoder = PcmCodec::from_stream(stream_reader)?;

    let dst_path = Path::new("./examples/samples/sine_copy.wav");
    let dst_file = File::create(dst_path)?;

    let stream_writer = WavFormat::write(dst_file, wav_header)?.into_dyn_stream()?;
    let mut encoder = PcmCodec::from_stream(stream_writer)?;

    let mut buf = [i16::MID; 1024];
    pipe_buffered(&mut decoder, &mut encoder, &mut buf)
}

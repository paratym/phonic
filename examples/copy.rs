use std::{fs::File, path::Path};
use syphon::{
    io::{
        codecs::PcmCodec,
        formats::WavFormat,
        utils::{pipe_buffered, Track},
    },
    Sample, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/samples/sine.wav");
    let src_file = File::open(src_path)?;

    let format_reader = WavFormat::reader(src_file)?;
    let wav_header = *format_reader.header();

    let track_reader = Track::default_reader(format_reader.into_dyn_format())?;
    let mut decoder = PcmCodec::decoder(track_reader)?;

    let dst_path = Path::new("./examples/samples/sine_copy.wav");
    let dst_file = File::create(dst_path)?;

    let format_writer = WavFormat::writer(dst_file, wav_header)?.into_dyn_format();
    let track_writer = Track::default_writer(format_writer)?;
    let mut encoder = PcmCodec::encoder(track_writer)?;

    let mut buf = [i16::MID; 1024];
    pipe_buffered(&mut decoder, &mut encoder, &mut buf)
}

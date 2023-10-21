use std::{fs::File, path::Path};
use syphon::{
    io::{
        codecs::PcmCodec,
        formats::WavFormat,
        utils::{pipe_buffered, TrackReader},
        EncodedStreamReader, SampleReader,
    },
    Sample, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/samples/sine.wav");
    let src_file = File::open(src_path)?;

    let format_reader = WavFormat::reader(src_file)?.into_format_reader();
    let track_reader = TrackReader::default(format_reader)?;
    let stream_spec = *track_reader.stream_spec();
    let mut decoder = PcmCodec::from_encoded_stream(track_reader)?;

    let dst_path = Path::new("./examples/samples/sine_copy.wav");
    let dst_file = File::create(dst_path)?;

    let format_writer = WavFormat::writer(dst_file, stream_spec.try_into()?)?;
    let mut encoder = PcmCodec::new(format_writer, *SampleReader::<i16>::stream_spec(&decoder));

    let mut buf = [i16::MID; 1024];
    pipe_buffered(&mut decoder, &mut encoder, &mut buf)
}

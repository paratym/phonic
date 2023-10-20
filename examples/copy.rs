use std::{fs::File, path::Path};
use syphon::{
    io::{
        formats::WavFormat, syphon_codec_registry, syphon_format_registry, utils::TrackReader,
        FormatIdentifier, SampleReaderRef, FormatReader,
    },
    Sample, SyphonError,
};

fn main() -> Result<(), SyphonError> {
    let src_path = Path::new("./examples/samples/sine.wav");
    let src_file = File::open(src_path)?;
    let identifier = src_path
        .extension()
        .map(|ext| ext.to_str())
        .flatten()
        .map(|ext| FormatIdentifier::FileExtension(ext));

    let format_reader = WavFormat::reader(src_file)?.into_format_reader();
    // let mut format_reader = syphon_format_registry().resolve_reader(src_file, identifier)?;

    let track_reader = Box::new(TrackReader::default(format_reader)?);
    // let mut decoder = PcmDecoder::new(track_reader)?;
    let mut decoder = match syphon_codec_registry().construct_decoder(track_reader)? {
        SampleReaderRef::I16(reader) => reader,
        _ => return Err(SyphonError::Unsupported),
    };

    let mut buf = [i16::MID; 1024];

    loop {
        let n = decoder.read(&mut buf)?;
        if n == 0 {
            break;
        }

        println!("{:?}", &buf[..n]);
    }

    Ok(())
}

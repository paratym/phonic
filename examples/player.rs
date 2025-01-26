use cpal::{
    traits::{HostTrait, StreamTrait},
    BufferSize, SizedSample,
};
use phonic::{
    cpal::DeviceExt,
    io::{
        dyn_io::{DynFormatConstructor, DynStream, FormatIdentifier, KnownSample},
        match_tagged_signal,
        utils::FormatUtilsExt,
    },
    sync::spsc::SpscSignal,
    utils::SignalUtilsExt,
    BlockingSignal, PhonicError, PhonicResult, SignalExt, SignalReader,
};
use std::{fs::File, path::Path, time::Duration};

fn main() -> PhonicResult<()> {
    // let path_arg = std::env::args().nth(1).expect("missing file arg");
    let path_arg = "/home/ben/Downloads/file_example_WAV_1MG.wav";
    let path = Path::new(path_arg);
    let file = File::open(path)?;

    let format = FormatIdentifier::try_from(path)?
        .known_format()
        .ok_or(PhonicError::Unsupported)?
        .read_index(file)?;

    let signal = format.into_primary_stream()?.into_decoder()?;
    match_tagged_signal!(signal, inner => play(inner))
}

fn play<S>(mut signal: S) -> PhonicResult<()>
where
    S: BlockingSignal + SignalReader + Send + Sync,
    S::Sample: KnownSample + SizedSample,
{
    let spec = signal.spec();
    const BUF_DURATION: Duration = Duration::from_millis(200);
    let (mut producer, consumer) = SpscSignal::default_duration(*spec, BUF_DURATION);

    let output = cpal::default_host()
        .default_output_device()
        .expect("no default output device")
        .build_output_stream_from_signal(
            consumer,
            |e| panic!("output error: {e}"),
            BufferSize::Default,
            None,
        );

    output.unwrap().play().unwrap();
    (&mut producer).polled().copy_all_buffered(&mut signal)?;
    producer.polled().flush_blocking()
}

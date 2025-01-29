use cpal::{
    traits::{HostTrait, StreamTrait},
    SizedSample,
};
use phonic::{
    cpal::CpalSignal,
    io::{
        dynamic::{DynFormatConstructor, DynStream, FormatIdentifier, KnownSample},
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
        .ok_or(PhonicError::unsupported())?
        .read_index(file)?;

    let signal = format.into_primary_stream()?.into_decoder()?;
    match_tagged_signal!(signal, inner => play(inner))
}

fn play<S>(signal: S) -> PhonicResult<()>
where
    S: BlockingSignal + SignalReader + Send + Sync,
    S::Sample: KnownSample + SizedSample,
{
    let spec = signal.spec();
    const BUF_DURATION: Duration = Duration::from_millis(200);
    let (mut producer, consumer) = SpscSignal::default_duration(*spec, BUF_DURATION);

    let device = cpal::default_host().default_output_device().unwrap();
    let output = <CpalSignal>::new().build_output(&device, consumer);
    output.unwrap().play().unwrap();

    (&mut producer).polled().copy_all_buffered(signal)?;
    producer.polled().flush_blocking()
}

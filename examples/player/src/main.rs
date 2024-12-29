use cpal::{
    traits::{HostTrait, StreamTrait},
    BufferSize, SizedSample,
};
use phonic::{
    buf::spsc::SignalBuf,
    cpal::DeviceExt,
    io::{
        match_tagged_signal,
        utils::{FormatIdentifier, FormatUtilsExt},
        DynFormatConstructor, DynStream, KnownFormat, KnownSample, TaggedSignal,
    },
    utils::SignalUtilsExt,
    BlockingSignalReader, PhonicError, PhonicResult,
};
use std::{env, fmt::Debug, fs::File, path::Path, thread::sleep, time::Duration};

fn main() -> PhonicResult<()> {
    // let path_arg = env::args().nth(1).ok_or(PhonicError::MissingData)?;
    let path_arg = "/home/ben/Desktop/phonic/examples/export/sine.wav";
    let path = Path::new(path_arg);
    let file = File::open(path)?;

    let format = KnownFormat::try_from(FormatIdentifier::try_from(path)?)?;
    let signal = format
        .read_index(file)?
        .into_primary_stream()?
        .into_decoder()?;

    match_tagged_signal!(signal, inner => play(inner))
}

fn play<S>(mut signal: S) -> PhonicResult<()>
where
    S: BlockingSignalReader + Send + Sync,
    S::Sample: KnownSample + SizedSample,
{
    let spec = signal.spec();
    const BUF_DURATION: Duration = Duration::from_millis(200);
    let (mut producer, consumer) = SignalBuf::default_duration(*spec, BUF_DURATION);

    let output = cpal::default_host()
        .default_output_device()
        .ok_or(PhonicError::NotFound)?
        .build_output_stream_from_signal(
            consumer,
            |e| panic!("output error: {e}"),
            BufferSize::Default,
            None,
        );

    output.unwrap().play().unwrap();
    producer.copy_all_buffered(&mut signal)?;

    sleep(Duration::from_secs(1));
    // producer.flush_blocking()

    Ok(())
}

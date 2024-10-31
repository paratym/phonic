use cpal::{
    traits::{HostTrait, StreamTrait},
    SizedSample,
};
use phonic::{
    cpal::DeviceExt,
    io::{
        match_tagged_signal, utils::FormatIdentifier, DynFormatConstructor, DynStream, Format,
        KnownFormat, KnownSample, TaggedSignal,
    },
    rtrb::RealTimeSignal,
    signal::{SignalReader, SignalWriter},
    PhonicError,
};
use std::{env, fs::File, path::Path, time::Duration};

fn main() -> Result<(), PhonicError> {
    let path_arg = env::args().nth(1).ok_or(PhonicError::MissingData)?;
    let path = Path::new(path_arg.as_str());
    let file = File::open(path)?;

    let format = KnownFormat::try_from(&FormatIdentifier::try_from(path)?)?;
    let signal = format
        .read_index(file)?
        .into_default_stream()?
        .into_decoder()?;

    match_tagged_signal!(signal, inner => play(inner))
}

const BUF_DURATION: Duration = Duration::from_millis(200);

fn play<S>(mut signal: S) -> Result<(), PhonicError>
where
    S: SignalReader + Send + Sync,
    S::Sample: KnownSample + SizedSample + Send + 'static,
{
    let spec = signal.spec();
    let buf_cap =
        spec.sample_rate_interleaved() as usize * BUF_DURATION.as_millis() as usize / 1000;

    let (mut producer, consumer) = RealTimeSignal::new(buf_cap, *spec);

    let output = cpal::default_host()
        .default_output_device()
        .ok_or(PhonicError::IoError)?
        .build_output_stream_from_signal(consumer, |e| panic!("output error: {e}"), None)
        .map_err(|_| PhonicError::IoError)?;

    output.play().map_err(|_| PhonicError::IoError)?;
    producer.copy_all(&mut signal, true)?;

    std::thread::sleep(BUF_DURATION);
    output.pause().map_err(|_| PhonicError::IoError)
}

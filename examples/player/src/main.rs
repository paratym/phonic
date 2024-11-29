use cpal::{
    traits::{HostTrait, StreamTrait},
    BufferSize, SizedSample,
};
use phonic::{
    cpal::DeviceExt,
    io::{
        match_tagged_signal, utils::FormatIdentifier, DynFormatConstructor, DynStream, Format,
        KnownFormat, KnownSample, TaggedSignal,
    },
    rtrb::SignalBuffer,
    signal::{utils::PollSignalWriter, PhonicError, PhonicResult, SignalReader, SignalWriter},
};
use std::{env, fs::File, path::Path, time::Duration};

fn main() -> PhonicResult<()> {
    let path_arg = env::args().nth(1).ok_or(PhonicError::MissingData)?;
    let path = Path::new(path_arg.as_str());
    let file = File::open(path)?;

    let format = KnownFormat::try_from(FormatIdentifier::try_from(path)?)?;
    let signal = format
        .read_index(file)?
        .into_default_stream()?
        .into_decoder()?;

    match_tagged_signal!(signal, inner => play(inner))
}

const BUF_DURATION: Duration = Duration::from_millis(200);

fn play<S>(mut signal: S) -> PhonicResult<()>
where
    S: SignalReader + Send + Sync,
    S::Sample: KnownSample + SizedSample + Send + 'static,
{
    let spec = signal.spec();
    let (mut producer, consumer) = SignalBuffer::new_duration(*spec, BUF_DURATION);

    let output = cpal::default_host()
        .default_output_device()
        .ok_or(PhonicError::NotFound)?
        .build_output_stream_from_signal(
            consumer,
            |e| panic!("output error: {e}"),
            BufferSize::Default,
            None,
        )
        .unwrap();

    output.play().unwrap();
    producer.copy_all(&mut signal)?;
    producer.flush()
}

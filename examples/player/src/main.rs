use cpal::{
    traits::{HostTrait, StreamTrait},
    SizedSample,
};
use phonic::{
    cpal::DeviceExt,
    io::{
        match_tagged_signal, utils::FormatIdentifier, DynFormatConstructor, DynStream, Format,
        FormatReader, KnownFormat, KnownSample, TaggedSignal,
    },
    rtrb::RingBufferHalfExt,
    signal::{SignalReader, SignalWriter},
    PhonicError,
};
use rtrb::RingBuffer;
use std::{env, fs::File, path::Path, thread::sleep, time::Duration};

fn main() -> Result<(), PhonicError> {
    let path_arg = env::args().nth(0).ok_or(PhonicError::MissingData)?;
    let path = Path::new(path_arg.as_str());
    let file = File::open(path)?;

    let signal = KnownFormat::try_from(&FormatIdentifier::try_from(path)?)?
        .from_std_io(file)?
        .with_reader_data()?
        .into_default_stream()?
        .into_codec()?;

    match_tagged_signal!(signal, inner => play(inner))
}

const BUF_DURATION: Duration = Duration::from_millis(200);

fn play<S>(mut signal: S) -> Result<(), PhonicError>
where
    S: SignalReader + Send + Sync,
    S::Sample: Default + KnownSample + SizedSample + Send + 'static,
{
    let spec = signal.spec();
    let buf_cap = BUF_DURATION.as_millis() as usize * spec.frame_rate as usize / 1000;
    let (producer, consumer) = RingBuffer::<S::Sample>::new(buf_cap);

    let output = cpal::default_host()
        .default_output_device()
        .ok_or(PhonicError::NotFound)?
        .build_output_stream_from_signal(
            consumer.into_signal(*spec),
            |e| panic!("output stream error: {e}"),
            None,
        )
        .map_err(|_| PhonicError::IoError)?;

    let mut output_signal = producer.into_signal(*spec);
    output_signal.copy_n(&mut signal, buf_cap as u64)?;

    output.play().map_err(|_| PhonicError::IoError)?;

    loop {
        match output_signal.copy_all(&mut signal) {
            Ok(_) | Err(PhonicError::OutOfBounds) => break,
            Err(PhonicError::Interrupted) => continue,
            Err(PhonicError::NotReady) => sleep(BUF_DURATION / 20),
            Err(e) => return Err(e),
        }
    }

    sleep(BUF_DURATION);
    Ok(())
}

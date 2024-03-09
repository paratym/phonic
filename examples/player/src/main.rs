use cpal::{
    traits::{HostTrait, StreamTrait},
    SizedSample,
};
use rtrb::RingBuffer;
use std::{env, fs::File, path::Path, thread::sleep, time::Duration};
use syphon::{
    cpal::DeviceExt,
    io::{
        match_tagged_signal,
        utils::{FormatIdentifier, TaggedSignal},
        DynFormatConstructor, DynStream, Format, KnownFormat,
    },
    rtrb::RingBufferHalfExt,
    signal::{KnownSample, SignalReader, SignalSpec, SignalWriter},
    synth::generators::SineGenerator,
    SyphonError,
};

fn main() -> Result<(), SyphonError> {
    // let path_arg = env::args().nth(0).ok_or(SyphonError::MissingData)?;
    // let path = Path::new(path_arg.as_str());
    // let file = File::open(path)?;
    //
    // let signal = KnownFormat::try_from(&FormatIdentifier::try_from(path)?)?
    //     .from_std_io(file)?
    //     .into_default_stream()?
    //     .into_codec()?;
    //
    // match_tagged_signal!(signal, inner => play(inner))

    let spec = SignalSpec::builder()
        .with_channels(2)
        .with_frame_rate(44100)
        .with_duration(Duration::from_secs(1))
        .build()?;

    let signal = SineGenerator::new(spec, 440.0);
    play(signal)
}

const BUF_DURATION: Duration = Duration::from_millis(200);

fn play<S>(mut signal: S) -> Result<(), SyphonError>
where
    S: SignalReader + Send + Sync,
    S::Sample: Default + KnownSample + SizedSample + Send + 'static,
{
    let spec = signal.spec();
    let buf_cap = BUF_DURATION.as_millis() as usize * spec.frame_rate as usize / 1000;
    let (producer, consumer) = RingBuffer::<S::Sample>::new(buf_cap);

    let output = cpal::default_host()
        .default_output_device()
        .ok_or(SyphonError::NotFound)?
        .build_output_stream_from_signal(
            consumer.into_signal(*spec),
            |e| panic!("output stream error: {e}"),
            None,
        )
        .map_err(|_| SyphonError::IoError)?;

    let mut output_signal = producer.into_signal(*spec);
    output_signal.copy_n(&mut signal, buf_cap as u64)?;

    output.play().map_err(|_| SyphonError::IoError)?;

    loop {
        match output_signal.copy_all(&mut signal) {
            Ok(_) | Err(SyphonError::EndOfStream) => break,
            Err(SyphonError::Interrupted) => continue,
            Err(SyphonError::NotReady) => sleep(BUF_DURATION / 20),
            Err(e) => return Err(e),
        }
    }

    sleep(BUF_DURATION);
    Ok(())
}

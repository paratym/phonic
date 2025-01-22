use crate::{
    utils::{IntoDuration, NSamples},
    BlockingSignal, BufferedSignalWriter, PhonicError, PhonicResult, Signal, SignalExt,
    SignalReader, SignalWriter,
};
use std::mem::MaybeUninit;

pub fn copy_exact<R, W>(
    reader: &mut R,
    writer: &mut W,
    duration: impl IntoDuration<NSamples>,
    buf: &mut [MaybeUninit<R::Sample>],
) -> PhonicResult<()>
where
    R: BlockingSignal + SignalReader,
    W: BlockingSignal + SignalWriter<Sample = R::Sample>,
{
    let spec = reader.spec().merged(writer.spec())?;
    let NSamples { n_samples } = duration.into_duration(&spec);
    let mut n = 0;

    while n < n_samples {
        let len = buf.len().min((n_samples - n) as usize);
        let samples = match reader.read_init_blocking(&mut buf[..len]) {
            Err(PhonicError::Interrupted | PhonicError::NotReady) => continue,
            Err(e) => return Err(e),
            Ok([]) => return Err(PhonicError::OutOfBounds),
            Ok(samples) => samples,
        };

        writer.write_exact(samples)?;
        n += samples.len() as u64;
    }

    Ok(())
}

pub fn copy_exact_buffered<R, W>(
    reader: &mut R,
    writer: &mut W,
    duration: impl IntoDuration<NSamples>,
) -> PhonicResult<()>
where
    R: BlockingSignal + SignalReader,
    W: BlockingSignal + BufferedSignalWriter<Sample = R::Sample>,
{
    let spec = reader.spec().merged(writer.spec())?;
    let n_channels = spec.channels.count() as usize;
    let n_samples = IntoDuration::<NSamples>::into_duration(duration, &spec).n_samples as usize;
    let mut n = 0;

    while n < n_samples {
        let Some(buf) = writer.buffer_mut() else {
            return Err(PhonicError::OutOfBounds);
        };

        if buf.len() < n_channels {
            writer.flush_blocking()?;
            continue;
        }

        let len = buf.len().min(n_samples - n);
        let n_read = match reader.read_blocking(&mut buf[..len]) {
            Err(PhonicError::Interrupted | PhonicError::NotReady) => continue,
            Err(e) => return Err(e),
            Ok(0) => return Err(PhonicError::OutOfBounds),
            Ok(n_read) => n_read,
        };

        writer.commit(n_read);
        n += n_read;
    }

    Ok(())
}

pub fn copy_all<R, W>(
    reader: &mut R,
    writer: &mut W,
    buf: &mut [MaybeUninit<R::Sample>],
) -> PhonicResult<()>
where
    R: BlockingSignal + SignalReader,
    W: BlockingSignal + SignalWriter<Sample = R::Sample>,
{
    let _spec = reader.spec().merged(writer.spec())?;

    loop {
        let samples = match reader.read_init_blocking(buf) {
            Err(PhonicError::Interrupted | PhonicError::NotReady) => continue,
            Err(e) => return Err(e),
            Ok([]) => break,
            Ok(samples) => samples,
        };

        match writer.write_exact(samples) {
            Ok(()) => continue,
            Err(PhonicError::OutOfBounds) => break,
            Err(e) => return Err(e),
        };
    }

    Ok(())
}

pub fn copy_all_buffered<R, W>(reader: &mut R, writer: &mut W) -> PhonicResult<()>
where
    R: BlockingSignal + SignalReader,
    W: BlockingSignal + BufferedSignalWriter<Sample = R::Sample>,
{
    let spec = reader.spec().merged(writer.spec())?;
    let n_channels = spec.channels.count() as usize;

    loop {
        let Some(buf) = writer.buffer_mut() else {
            break;
        };

        if buf.len() < n_channels {
            writer.flush_blocking()?;
            continue;
        }

        match reader.read_blocking(buf) {
            Ok(0) => break,
            Ok(n) => writer.commit(n),
            Err(PhonicError::Interrupted | PhonicError::NotReady) => continue,
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

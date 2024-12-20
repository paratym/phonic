use crate::{
    BlockingSignalReader, BlockingSignalWriter, NSamples, PhonicError, PhonicResult, Signal,
    SignalDuration,
};
use std::mem::MaybeUninit;

pub fn copy_exact<R, W, D>(
    reader: &mut R,
    writer: &mut W,
    duration: D,
    buf: &mut [MaybeUninit<R::Sample>],
) -> PhonicResult<()>
where
    R: BlockingSignalReader,
    W: BlockingSignalWriter<Sample = R::Sample>,
    D: SignalDuration,
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

pub fn copy_all<R, W>(
    reader: &mut R,
    writer: &mut W,
    buf: &mut [MaybeUninit<R::Sample>],
) -> PhonicResult<()>
where
    R: BlockingSignalReader,
    W: BlockingSignalWriter<Sample = R::Sample>,
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

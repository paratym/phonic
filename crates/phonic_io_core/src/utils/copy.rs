use crate::{BlockingStreamReader, BlockingStreamWriter, NBytes, StreamDuration};
use phonic_signal::{PhonicError, PhonicResult};
use std::mem::MaybeUninit;

pub fn copy_stream_exact<R, W, D>(
    reader: &mut R,
    writer: &mut W,
    duration: D,
    buf: &mut [MaybeUninit<u8>],
) -> PhonicResult<()>
where
    R: BlockingStreamReader,
    W: BlockingStreamWriter,
    D: StreamDuration,
    W::Tag: TryInto<R::Tag>,
    PhonicError: From<<W::Tag as TryInto<R::Tag>>::Error>,
{
    let spec = reader.stream_spec().merged(writer.stream_spec())?;
    let NBytes { n_bytes } = duration.into_stream_duration(&spec);
    debug_assert!(
        n_bytes % spec.block_align as u64 == 0,
        "n bytes not divisible by block align"
    );

    let mut n = 0;
    while n < n_bytes {
        let len = buf.len().min((n_bytes - n) as usize);
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

pub fn copy_stream_all<R, W>(
    reader: &mut R,
    writer: &mut W,
    buf: &mut [MaybeUninit<u8>],
) -> PhonicResult<()>
where
    R: BlockingStreamReader,
    W: BlockingStreamWriter,
    W::Tag: TryInto<R::Tag>,
    PhonicError: From<<W::Tag as TryInto<R::Tag>>::Error>,
{
    let _spec = reader.stream_spec().merged(writer.stream_spec())?;

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

use crate::{Sample, SignalReader, SignalWriter};
use syphon_core::SyphonError;

pub fn copy<S: Sample>(
    reader: &mut impl SignalReader<Sample = S>,
    writer: &mut impl SignalWriter<Sample = S>,
    mut buffer: &mut [S],
) -> Result<u64, SyphonError> {
    let spec = reader.spec();
    if spec != writer.spec() {
        return Err(SyphonError::SignalMismatch);
    }

    let mut n_read = 0;
    loop {
        let n = match reader.read(&mut buffer) {
            Ok(0) | Err(SyphonError::EndOfStream) => return Ok(n_read),
            Ok(n) => n,
            Err(SyphonError::Interrupted) | Err(SyphonError::NotReady) => continue,
            Err(e) => return Err(e),
        };

        writer.write_exact(&buffer[..n])?;
        n_read += n as u64;
    }
}

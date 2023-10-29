use crate::{Sample, SignalReader, SignalWriter, SyphonError};

pub fn copy<S: Sample>(
    reader: &mut dyn SignalReader<S>,
    writer: &mut dyn SignalWriter<S>,
    mut buffer: &mut [S],
) -> Result<(), SyphonError> {
    let spec = reader.spec();
    let samples_per_block = spec.samples_per_block();
    if spec.sample_format != S::FORMAT || spec != writer.spec() {
        return Err(SyphonError::SignalMismatch);
    }

    let mut n;
    loop {
        n = match reader.read(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(n) => n * samples_per_block,
            Err(SyphonError::EndOfStream) => return Ok(()),
            Err(SyphonError::Interrupted) | Err(SyphonError::NotReady) => continue,
            Err(e) => return Err(e),
        };

        writer.write_exact(&buffer[..n])?;
    }
}

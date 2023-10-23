use crate::{Sample, SampleReader, SampleWriter, SyphonError};

pub fn pipe_buffered<S: Sample>(
    reader: &mut dyn SampleReader<S>,
    writer: &mut dyn SampleWriter<S>,
    buffer: &mut [S],
) -> Result<(), SyphonError> {
    let spec = reader.spec();
    let buf_len = buffer.len() - buffer.len() % spec.block_size;
    if S::FORMAT != spec.sample_format || writer.spec() != spec {
        return Err(SyphonError::StreamMismatch);
    }

    let mut n;
    loop {
        n = match reader.read(&mut buffer[..buf_len]) {
            Ok(0) => return Ok(()),
            Ok(n) => n,
            Err(SyphonError::EndOfStream) => return Ok(()),
            Err(SyphonError::Interrupted) | Err(SyphonError::NotReady) => continue,
            Err(e) => return Err(e),
        };

        writer.write_exact(&buffer[..n])?;
    }
}

pub fn pipe<S: Sample>(
    reader: &mut dyn SampleReader<S>,
    writer: &mut dyn SampleWriter<S>,
) -> Result<(), SyphonError> {
    if reader.spec() != writer.spec() {
        return Err(SyphonError::StreamMismatch);
    }

    let mut buffer = vec![S::MID; reader.spec().block_size as usize];
    pipe_buffered(reader, writer, buffer.as_mut_slice())
}

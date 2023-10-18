use crate::{
    io::{SampleReader, SampleWriter},
    Sample, SyphonError,
};

pub fn pipe_buffered<S: Sample>(
    reader: &mut dyn SampleReader<S>,
    writer: &mut dyn SampleWriter<S>,
    buffer: &mut [S],
) -> Result<(), SyphonError> {
    let spec = *reader.stream_spec();
    if writer.stream_spec() != &spec {
        return Err(SyphonError::StreamMismatch);
    } else if buffer.len() % spec.block_size != 0 {
        return Err(SyphonError::StreamMismatch);
    }

    loop {
        match reader.read(buffer) {
            Ok(0) => return Ok(()),
            Ok(n) if n % spec.block_size == 0 => {}
            Ok(_) => return Err(SyphonError::MalformedData),
            Err(SyphonError::Empty) => return Ok(()),
            Err(e) => return Err(e),
        }

        writer.write_exact(buffer)?;
    }
}

pub fn pipe<S: Sample>(
    reader: &mut dyn SampleReader<S>,
    writer: &mut dyn SampleWriter<S>,
) -> Result<(), SyphonError> {
    if reader.stream_spec() != writer.stream_spec() {
        return Err(SyphonError::StreamMismatch);
    }

    let mut buffer = vec![S::MID; reader.stream_spec().block_size as usize];
    pipe_buffered(reader, writer, buffer.as_mut_slice())
}

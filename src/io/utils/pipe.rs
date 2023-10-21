use crate::{
    io::{SampleReader, SampleWriter},
    Sample, SyphonError,
};

pub fn pipe_buffered<S: Sample>(
    reader: &mut dyn SampleReader<S>,
    writer: &mut dyn SampleWriter<S>,
    buffer: &mut [S],
) -> Result<(), SyphonError> {
    let spec = reader.stream_spec();
    let buf_len = buffer.len() - buffer.len() % spec.block_size;
    if writer.stream_spec() != spec {
        return Err(SyphonError::StreamMismatch);
    }

    let mut n;
    loop {
        n = match reader.read(&mut buffer[..buf_len]) {
            Ok(0) => return Ok(()),
            Ok(n) => n,
            Err(SyphonError::Empty) => return Ok(()),
            Err(e) => return Err(e),
        };

        writer.write_exact(&buffer[..n])?;
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

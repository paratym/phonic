use crate::{
    io::{SampleReader, StreamSpec},
    Sample, SyphonError,
};

pub struct BlockSizeAdapter<S: Sample> {
    source: Box<dyn SampleReader<S>>,
    stream_spec: StreamSpec,
    buffer: Box<[S]>,
    block_size: usize,
    n_buffered: usize,
    n_read: usize,
}

impl<S: Sample> BlockSizeAdapter<S> {
    pub fn new(
        source: impl SampleReader<S> + 'static,
        buffer: Box<[S]>,
        n_frames: usize,
    ) -> Result<Self, SyphonError> {
        let src_spec = source.stream_spec();
        let src_block_size = src_spec.block_size as usize;
        let block_size = n_frames * src_spec.n_channels as usize;

        if buffer.len() < src_block_size || buffer.len() < block_size {
            return Err(SyphonError::StreamMismatch);
        }

        let stream_spec = StreamSpec {
            block_size,
            ..*src_spec
        };

        Ok(Self {
            source: Box::new(source),
            stream_spec,
            buffer,
            block_size,
            n_buffered: 0,
            n_read: 0,
        })
    }
}

impl<S: Sample> SampleReader<S> for BlockSizeAdapter<S> {
    fn stream_spec(&self) -> &StreamSpec {
        &self.stream_spec
    }

    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

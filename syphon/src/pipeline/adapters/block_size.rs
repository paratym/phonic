use crate::core::{Sample, SampleReader, SignalSpec, SyphonError};

pub struct BlockSizeAdapter<S: Sample> {
    source: Box<dyn SampleReader<S>>,
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
        let src_block_size = source.signal_spec().block_size as usize;
        let block_size = n_frames * source.signal_spec().n_channels as usize;

        if buffer.len() < src_block_size || buffer.len() < block_size {
            return Err(SyphonError::SignalMismatch);
        }

        Ok(Self {
            source: Box::new(source),
            buffer,
            block_size,
            n_buffered: 0,
            n_read: 0,
        })
    }
}

impl<S: Sample> SampleReader<S> for BlockSizeAdapter<S> {
    fn signal_spec(&self) -> SignalSpec {
        SignalSpec {
            block_size: self.block_size,
            ..self.source.signal_spec()
        }
    }

    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

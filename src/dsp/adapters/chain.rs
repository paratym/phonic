use std::io::{Seek, SeekFrom};

use crate::{Sample, Signal, SignalReader, SignalSpec, SignalWriter, SyphonError};

pub struct SignalChain<A: Signal<S>, B: Signal<S>, S: Sample> {
    first: A,
    second: B,
    spec: SignalSpec<S>,
}

impl<A: Signal<S>, B: Signal<S>, S: Sample> SignalChain<A, B, S> {
    pub fn new(first: A, second: B) -> Self {
        let mut spec = *first.spec();
        spec.n_blocks = spec
            .n_blocks
            .zip(second.spec().n_blocks)
            .map(|(a, b)| a + b);

        // TODO: check that the specs are compatible

        Self {
            first,
            second,
            spec,
        }
    }
}

impl<A: Signal<S>, B: Signal<S>, S: Sample> Signal<S> for SignalChain<A, B, S> {
    fn spec(&self) -> &SignalSpec<S> {
        &self.spec
    }
}

impl<A, B, S> SignalReader<S> for SignalChain<A, B, S>
where
    A: SignalReader<S>,
    B: SignalReader<S>,
    S: Sample,
{
    fn read(&mut self, buf: &mut [S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<A, B, S> SignalWriter<S> for SignalChain<A, B, S>
where
    A: SignalWriter<S>,
    B: SignalWriter<S>,
    S: Sample,
{
    fn write(&mut self, buf: &[S]) -> Result<usize, SyphonError> {
        todo!()
    }
}

impl<A, B, S> Seek for SignalChain<A, B, S>
where
    A: Signal<S> + Seek,
    B: Signal<S> + Seek,
    S: Sample,
{
    fn seek(&mut self, offset: SeekFrom) -> std::io::Result<u64> {
        todo!()
    }
}

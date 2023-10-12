use crate::{
    core::{SignalSpecBuilder, SyphonError},
    io::{AudioFormatReader, FormatCodecId},
};
use std::io::Read;

pub struct WavReader {
    reader: Box<dyn Read>,
}

impl WavReader {
    pub fn new(reader: Box<dyn Read>) -> Self {
        Self { reader }
    }
}

pub struct WavCodecId(pub u16);
impl FormatCodecId for WavCodecId {}

impl AudioFormatReader for WavReader {
    fn read_spec(&mut self) -> Result<(Box<dyn FormatCodecId>, SignalSpecBuilder), SyphonError> {
        let spec_builder = SignalSpecBuilder::new();

        Ok((Box::new(WavCodecId(0)), spec_builder))
    }
}

impl Read for WavReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

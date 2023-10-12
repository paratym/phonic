use crate::{core::SyphonError, io::{formats::FormatReader, SyphonCodecKey}};
use std::io::Read;
use super::FormatDataBuilder;

pub struct WavCodecId(pub u16);

impl TryFrom<WavCodecId> for SyphonCodecKey {
    type Error = SyphonError;

    fn try_from(WavCodecId(id): WavCodecId) -> Result<Self, Self::Error> {
        match id {
            _ => Err(SyphonError::Unsupported),
        }
    }
}

pub struct WavReader {
    reader: Box<dyn Read>,
}

impl WavReader {
    pub fn new(reader: Box<dyn Read>) -> Self {
        Self { reader }
    }
}

impl<K: TryFrom<WavCodecId>> FormatReader<K> for WavReader {
    fn read_spec(&mut self) -> Result<FormatDataBuilder<K>, SyphonError> {
        Ok(FormatDataBuilder::new())
    }
}

impl Read for WavReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

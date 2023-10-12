use crate::core::{SignalSpec, SignalSpecBuilder, SyphonError};
use std::io::{Read, Write};

pub struct FormatDataBuilder<K> {
    pub spec: SignalSpecBuilder,
    pub codec_key: Option<K>,
}

impl<K> FormatDataBuilder<K> {
    pub fn new() -> Self {
        Self {
            spec: SignalSpecBuilder::new(),
            codec_key: None,
        }
    }
}

pub trait FormatReader<K>: Read {
    fn read_spec(&mut self) -> Result<FormatDataBuilder<K>, SyphonError>;
}

pub trait FormatWriter: Write {
    fn write_spec(&mut self, spec: SignalSpec) -> Result<(), SyphonError>;
}

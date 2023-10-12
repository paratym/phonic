use crate::core::{SignalSpec, SignalSpecBuilder, SyphonError};
use std::io::{Read, Write};

pub trait FormatCodecId {}

pub trait AudioFormatReader: Read {
    fn read_spec(&mut self) -> Result<(Box<dyn FormatCodecId>, SignalSpecBuilder), SyphonError>;
}

pub trait AudioFormatWriter: Write {
    fn write_spec(&mut self, spec: SignalSpec) -> Result<(), SyphonError>;
}

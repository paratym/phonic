use crate::core::{Sample, SignalSpec};
use std::io::{Read, Write};

pub trait AudioFormatReader: Read + From<Box<dyn Read>> {
    fn read_format(&mut self) -> Result<(), ()>;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, ()>;
}

pub trait AudioFormatWriter: Write + From<Box<dyn Write>> {
    fn write_format(&mut self) -> Result<(), ()>;
    fn write(&mut self, buffer: &[u8]) -> Result<usize, ()>;
}

pub trait AudioDecoder<S: Sample>: From<Box<dyn Read>> {
    fn signal_spec(&self) -> SignalSpec;
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, ()>;
}

pub trait AudioEncoder<S: Sample>: From<Box<dyn Write>> {
    fn signal_spec(&self) -> SignalSpec;
    fn write(&mut self, buffer: &[S]) -> Result<usize, ()>;
}
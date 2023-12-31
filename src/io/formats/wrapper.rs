use crate::{
    io::{Format, FormatData, FormatReadResult, FormatReader, FormatWriter},
    SyphonError,
};
use std::io::{Read, Write};

pub struct FormatWrapper<T> {
    inner: T,
    data: FormatData,
}

impl<T> FormatWrapper<T> {
    pub fn new(inner: T, data: FormatData) -> Self {
        Self { inner, data }
    }
}

impl<T> Format for FormatWrapper<T> {
    fn format_data(&self) -> &FormatData {
        &self.data
    }
}

impl<T: Read> FormatReader for FormatWrapper<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError> {
        self.inner
            .read(buf)
            .map(|n| FormatReadResult { track: 0, n })
            .map_err(Into::into)
    }
}

impl<T: Write> FormatWriter for FormatWrapper<T> {
    fn write(&mut self, track_i: usize, buf: &[u8]) -> Result<usize, SyphonError> {
        if track_i != 0 {
            return Err(SyphonError::NotFound);
        }

        self.inner.write(buf).map_err(Into::into)
    }

    fn flush(&mut self) -> Result<(), SyphonError> {
        self.inner.flush().map_err(Into::into)
    }
}

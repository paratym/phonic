use crate::{StreamObserver, StreamReader, StreamSeeker, StreamWriter};
use std::io::{Read, Seek, SeekFrom, Write};
use phonic_core::PhonicError;

pub struct StdIoStream<T>(T);

impl<T> Read for StdIoStream<T>
where
    T: StreamReader,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf).map_err(Into::into)
    }
}

impl<T> Write for StdIoStream<T>
where
    T: StreamWriter,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf).map_err(Into::into)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush().map_err(Into::into)
    }
}

impl<T> Seek for StdIoStream<T>
where
    T: StreamSeeker + StreamObserver,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Current(offset) => {
                self.0.seek(offset)?;
                self.0.position().map_err(Into::into)
            }
            SeekFrom::Start(position) => {
                self.0.set_position(position)?;
                Ok(position)
            }
            SeekFrom::End(offset) => {
                let len = self.0.spec().n_bytes().ok_or(PhonicError::MissingData)?;
                let position = (len as i64 + offset) as u64;
                self.0.set_position(position)?;
                Ok(position)
            }
        }
    }
}

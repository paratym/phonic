use crate::{FiniteStream, IndexedStream, StreamReader, StreamSeeker, StreamWriter};
use phonic_core::PhonicError;
use std::io::{Read, Seek, SeekFrom, Write};

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
    T: StreamSeeker + IndexedStream + FiniteStream,
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        match pos {
            SeekFrom::Current(offset) => {
                self.0.seek(offset)?;
                Ok(self.0.pos())
            }
            SeekFrom::Start(position) => {
                self.0.set_pos(position)?;
                Ok(self.0.pos())
            }
            SeekFrom::End(offset) => {
                let position = self
                    .0
                    .len()
                    .checked_add_signed(offset)
                    .ok_or(PhonicError::OutOfBounds)?;

                self.0.set_pos(position)?;
                Ok(self.0.pos())
            }
        }
    }
}

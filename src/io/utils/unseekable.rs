use std::io::{self, Read, Seek, SeekFrom, Write};

pub struct Unseekable<T> {
    inner: T,
    i: u64,
}

impl<T> Unseekable<T> {
    pub fn new(inner: T) -> Self {
        Self { inner, i: 0 }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Read> Read for Unseekable<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.inner.read(buf);
        if let Ok(n) = res {
            self.i += n as u64;
        }

        res
    }
}

impl<T: Write> Write for Unseekable<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let res = self.inner.write(buf);
        if let Ok(n) = res {
            self.i += n as u64;
        }

        res
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T> Seek for Unseekable<T> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        match offset {
            SeekFrom::Current(0) => Ok(self.i),
            _ => Err(io::ErrorKind::Unsupported.into()),
        }
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.i as u64)
    }
}

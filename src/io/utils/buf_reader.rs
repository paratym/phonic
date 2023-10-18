use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, Seek, SeekFrom};

pub struct BufReader<const N: usize, R: Read> {
    reader: R,
    buf: [u8; N],
    n_buffered: usize,
    n_read: usize,
}

impl<const N: usize, R: Read> BufReader<N, R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: [0; N],
            n_read: 0,
            n_buffered: 0,
        }
    }

    pub fn into_inner(self) -> R {
        self.reader
    }

    pub fn fill_buffer(&mut self) -> IoResult<usize> {
        if self.n_buffered >= N {
            return Ok(self.n_buffered);
        }

        let n = self.reader.read(&mut self.buf[self.n_buffered..])?;
        self.n_buffered += n;
        Ok(self.n_buffered)
    }

    pub fn peek(&self) -> &[u8] {
        &self.buf[self.n_read..self.n_buffered]
    }

    pub fn commit_read(&mut self) {
        let (l, r) = self.buf.split_at_mut(self.n_read);
        let n = self.n_buffered - self.n_read;
        l[..n].copy_from_slice(&r[..n]);

        self.n_read = 0;
        self.n_buffered = n;
    }
}

impl<const N: usize, R: Read> Read for BufReader<N, R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        if self.n_read >= N {
            return Err(ErrorKind::OutOfMemory.into());
        }

        self.fill_buffer()?;
        let n = buf.len().min(self.n_buffered - self.n_read);
        buf[..n].copy_from_slice(&self.buf[self.n_read..self.n_read + n]);
        self.n_read += n;
        Ok(n)
    }
}

impl<const N: usize, R: Read> Seek for BufReader<N, R> {
    fn seek(&mut self, pos: SeekFrom) -> IoResult<u64> {
        let n_read = match pos {
            SeekFrom::Start(n) => n as i64,
            SeekFrom::Current(0) => return Ok(self.n_read as u64),
            SeekFrom::Current(n) => n + self.n_buffered as i64,
            SeekFrom::End(_) => return Err(ErrorKind::Unsupported.into()),
        };

        if n_read < 0 || n_read > self.n_buffered as i64 {
            return Err(IoError::new(ErrorKind::Other, "out of range"));
        }

        self.n_read = n_read as usize;
        return Ok(self.n_read as u64);
    }
}

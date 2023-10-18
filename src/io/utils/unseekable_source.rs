use std::io::{self, Read, Seek, SeekFrom};

pub struct UnseekableMediaSource<R: Read> {
    reader: R,
    n_read: usize,
}

impl<R: Read> UnseekableMediaSource<R> {
    pub fn new(reader: R) -> Self {
        Self { reader, n_read: 0 }
    }
}

impl<R: Read> Read for UnseekableMediaSource<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let res = self.reader.read(buf);
        if let Ok(n) = res {
            self.n_read += n;
        }

        res
    }
}

impl<R: Read> Seek for UnseekableMediaSource<R> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        match offset {
            SeekFrom::Current(0) => Ok(self.n_read as u64),
            _ => Err(io::ErrorKind::Unsupported.into()),
        }
    }
}

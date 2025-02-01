use phonic_signal::utils::{DefaultSizedBuf, SizedBuf};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};

pub(super) struct RiffChunk<T> {
    inner: T,
    id: [u8; 4],
    len: u32,
    pos: u32,
}

impl<T> RiffChunk<T> {
    pub fn read_new(mut inner: T) -> io::Result<Self>
    where
        T: Read,
    {
        let mut buf = [0u8; 8];
        inner.read_exact(&mut buf)?;

        let [id, len_bytes]: [[u8; 4]; 2] = unsafe { std::mem::transmute(buf) };
        let len = u32::from_le_bytes(len_bytes);

        Ok(Self {
            inner,
            id,
            len,
            pos: 0,
        })
    }

    pub fn write_new(mut inner: T, id: [u8; 4]) -> io::Result<Self>
    where
        T: Write,
    {
        let len: u32 = 0;
        let len_bytes = len.to_le_bytes();

        inner.write_all(&id)?;
        inner.write_all(&len_bytes)?;

        Ok(Self {
            inner,
            id,
            len,
            pos: 0,
        })
    }

    pub fn id(&self) -> [u8; 4] {
        self.id
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn pos(&self) -> u32 {
        self.pos
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn skip_remaining(&mut self) -> io::Result<()>
    where
        T: Read,
    {
        let mut buf = <DefaultSizedBuf<_>>::filled(0u8);

        while self.pos < self.len {
            match self.inner.read(&mut buf) {
                Ok(0) => {
                    return Err(io::Error::new(
                        ErrorKind::UnexpectedEof,
                        "reached end of file before expected chunk boundary",
                    ))
                }
                Ok(n) => self.pos += n as u32,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    pub fn update_header(&mut self) -> io::Result<()>
    where
        T: Write + Seek,
    {
        let pos = self.pos as i64;
        let len_bytes = self.len.to_le_bytes();
        let len_offset = -pos - len_bytes.len() as i64;

        self.inner.seek_relative(len_offset)?;
        self.inner.write_all(&len_bytes)?;
        self.inner.seek_relative(pos)?;

        Ok(())
    }
}

impl<T: Read> Read for RiffChunk<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let rem = self.len - self.pos;
        if rem == 0 {
            return Ok(0);
        }

        let buf_len = buf.len().min(rem as usize);
        let n = self.inner.read(&mut buf[..buf_len])?;
        self.pos += n as u32;

        Ok(n)
    }
}

impl<T: Write> Write for RiffChunk<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;

        self.pos += n as u32;
        self.len = self.len.max(self.pos);

        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for RiffChunk<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos_result = match pos {
            SeekFrom::Start(offset) => Some(offset),
            SeekFrom::Current(offset) => (self.pos as u64).checked_add_signed(offset),
            SeekFrom::End(offset) => (self.len as u64).checked_add_signed(offset),
        };

        let out_of_bounds_err = io::Error::new(
            ErrorKind::InvalidInput,
            "attempted to seek outside of the defined bounds of the chunk",
        );

        let Some(new_pos) = new_pos_result else {
            return Err(out_of_bounds_err);
        };

        if new_pos > self.len as u64 {
            return Err(out_of_bounds_err);
        }

        self.pos = new_pos as u32;
        Ok(new_pos)
    }
}

pub(super) fn update_nested_chunk_header<W: Write + Seek>(
    chunk: &mut RiffChunk<RiffChunk<W>>,
) -> io::Result<()> {
    chunk.inner.update_header()?;
    chunk.update_header()
}

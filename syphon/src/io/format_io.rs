use crate::{
    io::{MediaStreamReader, StreamSpec, StreamSpecBuilder},
    SyphonError,
};
use std::io::{self, Read, Seek, SeekFrom};

pub trait MediaSource: Read + Seek {}

impl<T: Read + Seek> MediaSource for T {}

pub struct UnseekableMediaSource<R: Read> {
    reader: R,
}

impl<R: Read> UnseekableMediaSource<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: Read> Read for UnseekableMediaSource<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl<R: Read> Seek for UnseekableMediaSource<R> {
    fn seek(&mut self, _: io::SeekFrom) -> io::Result<u64> {
        Err(io::ErrorKind::Unsupported.into())
    }
}

pub struct FormatReadResult {
    pub n_bytes: usize,
    pub track_i: usize,
}

pub trait FormatReader {
    fn tracks(&self) -> Box<dyn Iterator<Item = StreamSpecBuilder>>;
    fn read_headers(&mut self) -> Result<(), SyphonError>;
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError>;
    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError>;
}

pub struct TrackReader {
    reader: Box<dyn FormatReader>,
    stream_spec: StreamSpec,
    track_i: usize,
}

impl TrackReader {
    pub fn new(reader: Box<dyn FormatReader>, track_i: usize) -> Result<Self, SyphonError> {
        let stream_spec = reader
            .tracks()
            .nth(track_i)
            .ok_or(SyphonError::BadRequest)?
            .try_build()?;

        Ok(Self {
            reader,
            stream_spec,
            track_i,
        })
    }

    pub fn default(reader: Box<dyn FormatReader>) -> Result<Self, SyphonError> {
        let track_i = reader
            .tracks()
            .enumerate()
            .find(|(_, t)| t.codec_key.is_some())
            .map(|(i, _)| i)
            .ok_or(SyphonError::BadRequest)?;
        
        Self::new(reader, track_i)
    }

    pub fn stream_spec(&self) -> &StreamSpec {
        &self.stream_spec
    }
}

impl MediaStreamReader for TrackReader {
    fn stream_spec(&self) -> &StreamSpec {
        &self.stream_spec
    }
}

impl Read for TrackReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let result = self.reader.read(buf)?;
            if result.track_i == self.track_i {
                return Ok(result.n_bytes);
            }
        }
    }
}

impl Seek for TrackReader {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        Ok(self.reader.seek(offset)?)
    }
}

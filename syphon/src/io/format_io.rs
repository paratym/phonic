use std::io::{self, Read, Seek, SeekFrom};
use crate::{SyphonError, SampleFormat, Sample, io::{SignalSpec, SignalSpecBuilder, SignalReader}};

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

#[derive(Clone, Copy)]
pub struct TrackData<K> {
    pub codec_key: K,
    pub signal_spec: SignalSpec,
    pub n_frames: Option<usize>,
}

#[derive(Clone, Copy)]
pub struct TrackDataBuilder<K> {
    pub codec_key: Option<K>,
    pub signal_spec: SignalSpecBuilder,
    pub n_frames: Option<usize>,
}

pub struct FormatReadResult {
    pub n_bytes: usize,
    pub track_i: usize,
}

pub trait FormatReader {
    type CodecKey: Copy;

    fn tracks(&self) -> Box<dyn Iterator<Item = TrackDataBuilder<Self::CodecKey>>>;
    fn read_headers(&mut self) -> Result<(), SyphonError>;
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError>;
    fn seek(&mut self, offset: SeekFrom) -> Result<u64, SyphonError>;
}

impl<K: Copy> TrackDataBuilder<K> {
    pub fn new() -> Self {
        Self {
            codec_key: None,
            signal_spec: SignalSpecBuilder::new(),
            n_frames: None,
        }
    }

    pub fn from_other<O: TryFrom<K>>(other: &Self) -> TrackDataBuilder<O> {
        TrackDataBuilder {
            codec_key: other.codec_key.map(|key| key.try_into().ok()).flatten(),
            signal_spec: other.signal_spec,
            n_frames: other.n_frames,
        }
    }

    pub fn build(self) -> Result<TrackData<K>, SyphonError> {
        Ok(TrackData {
            codec_key: self.codec_key.ok_or(SyphonError::MalformedData)?,
            signal_spec: self.signal_spec.try_build()?,
            n_frames: self.n_frames,
        })
    }
}

pub struct TrackReader<'a, K> {
    reader: &'a mut dyn FormatReader<CodecKey = K>,
    track_data: TrackData<K>,
    track_i: usize,
}

impl<'a, K: Copy> TrackReader<'a, K> {
    pub fn new(reader: &'a mut dyn FormatReader<CodecKey = K>, track_i: usize) -> Result<Self, SyphonError> {
        let track_data = reader
            .tracks()
            .nth(track_i)
            .ok_or(SyphonError::MalformedData)?
            .build()?;

        Ok(Self {
            reader,
            track_data,
            track_i,
        })
    }

    pub fn default(reader: &'a mut dyn FormatReader<CodecKey = K>) -> Result<Self, SyphonError> {
        let track_data = reader
            .tracks()
            .find(|t| t.codec_key.is_some())
            .ok_or(SyphonError::MalformedData)?
            .build()?;

        Ok(Self {
            reader,
            track_data,
            track_i: 0,
        })
    }
}

impl<K: Copy> SignalReader for TrackReader<'_, K> {
    fn signal_spec(&self) -> &SignalSpec {
        &self.track_data.signal_spec
    }
}

impl<K: Copy> Read for TrackReader<'_, K> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let result = self.reader.read(buf)?;
            if result.track_i == self.track_i {
                return Ok(result.n_bytes);
            }
        }
    }
}

impl<K: Copy> Seek for TrackReader<'_, K> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        Ok(self.reader.seek(offset)?)
    }
}

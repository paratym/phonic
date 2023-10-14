use crate::core::{SignalSpec, SignalSpecBuilder, SyphonError};
use std::io::{self, Read, Seek, SeekFrom};

pub trait MediaSource: Read + Seek {}

pub struct UnseekableMediaSource<R: Read> {
    reader: R,
}

impl<T: Read + Seek> MediaSource for T {}

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

pub struct TrackData<K> {
    pub signal_spec: SignalSpec,
    pub codec_key: K,
    pub n_frames: Option<usize>,
    pub channel_map: Option<()>,
}

#[derive(Clone, Copy)]
pub struct TrackDataBuilder<K> {
    pub signal_spec: SignalSpecBuilder,
    pub codec_key: Option<K>,
    pub n_frames: Option<usize>,
    pub channel_map: Option<()>,
}

pub struct FormatReadResult {
    pub n_bytes: usize,
    pub track_i: usize,
}

pub trait FormatReader {
    type CodecKey: Copy;

    fn tracks(&self) -> Box<dyn Iterator<Item = TrackDataBuilder<Self::CodecKey>>>;
    fn read_headers(&mut self) -> Result<usize, SyphonError>;
    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError>;
    fn seek(&mut self, offset: SeekFrom) -> Result<usize, SyphonError>;
}

impl<K: Copy> TrackDataBuilder<K> {
    pub fn new() -> Self {
        Self {
            signal_spec: SignalSpecBuilder::new(),
            codec_key: None,
            n_frames: None,
            channel_map: None,
        }
    }

    pub fn from_other<O: TryFrom<K>>(other: &Self) -> TrackDataBuilder<O> {
        TrackDataBuilder {
            signal_spec: other.signal_spec,
            codec_key: other.codec_key.map(|key| key.try_into().ok()).flatten(),
            n_frames: other.n_frames,
            channel_map: other.channel_map,
        }
    }

    pub fn build(self) -> Result<TrackData<K>, SyphonError> {
        Ok(TrackData {
            signal_spec: self.signal_spec.try_build()?,
            codec_key: self.codec_key.ok_or(SyphonError::MalformedData)?,
            n_frames: self.n_frames,
            channel_map: self.channel_map,
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

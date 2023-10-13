use crate::core::{SignalSpec, SignalSpecBuilder, SyphonError};

pub struct TrackData<K> {
    pub signal_spec: SignalSpec,
    pub codec_key: K,
    pub n_frames: Option<usize>,
    pub channel_map: Option<()>,
}

pub struct TrackDataBuilder<K> {
    pub signal_spec: SignalSpecBuilder,
    pub codec_key: Option<K>,
    pub n_frames: Option<usize>,
    pub channel_map: Option<()>,
}

impl<K> TrackDataBuilder<K> {
    pub fn new() -> Self {
        Self {
            signal_spec: SignalSpecBuilder::new(),
            codec_key: None,
            n_frames: None,
            channel_map: None,
        }
    }

    pub fn try_build(self) -> Result<TrackData<K>, SyphonError> {
        Ok(TrackData {
            signal_spec: self.signal_spec.try_build()?,
            codec_key: self.codec_key.ok_or(SyphonError::MalformedData)?,
            n_frames: self.n_frames,
            channel_map: self.channel_map,
        })
    }
}
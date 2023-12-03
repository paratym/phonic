use crate::{dsp::adapters::{SampleTypeAdapter, FrameRateAdapter, ChannelsAdapter, BlockSizeAdapter, NBlocksAdapter, SignalChain}, Sample, SampleType, SyphonError};
use std::{ops::{BitAnd, BitOr, BitXor}, time::Duration};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Channels {
    Count(u32),
    Layout(ChannelLayout),
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ChannelLayout {
    mask: u32,
}

/// A set of parameters that describes a pcm signal
#[derive(Copy, Clone)]
pub struct SignalSpec<S> {
    pub sample_type: S,

    /// The number of samples per channel per second.
    pub frame_rate: u32,

    /// The layout of the channels in the signal.
    pub channels: Channels,

    /// The minimum number of frames that can be read or written at once.
    /// This does not need to be enforced by the consumer, but can be useful for sizing buffers.
    pub block_size: usize,

    /// The total number of sample blocks in the signal.
    pub n_blocks: Option<u64>,
}

#[derive(Clone, Copy)]
pub struct SignalSpecBuilder<S> {
    pub sample_type: Option<S>,
    pub frame_rate: Option<u32>,
    pub channels: Option<Channels>,
    pub block_size: Option<usize>,
    pub n_blocks: Option<u64>,
}

impl Channels {
    pub fn count(&self) -> u32 {
        match self {
            Self::Count(n) => *n,
            Self::Layout(layout) => layout.count(),
        }
    }

    pub fn layout(&self) -> Option<&ChannelLayout> {
        match self {
            Self::Count(_) => None,
            Self::Layout(layout) => Some(layout),
        }
    }
}

impl From<ChannelLayout> for Channels {
    fn from(layout: ChannelLayout) -> Self {
        Self::Layout(layout)
    }
}

impl ChannelLayout {
    pub const fn from_bits(mask: u32) -> Self {
        Self { mask }
    }

    pub const FRONT_LEFT: Self = Self::from_bits(1 << 0);
    pub const FRONT_RIGHT: Self = Self::from_bits(1 << 1);
    pub const FRONT_CENTRE: Self = Self::from_bits(1 << 2);
    pub const LFE1: Self = Self::from_bits(1 << 3);
    pub const REAR_LEFT: Self = Self::from_bits(1 << 4);
    pub const REAR_RIGHT: Self = Self::from_bits(1 << 5);
    pub const FRONT_LEFT_CENTRE: Self = Self::from_bits(1 << 6);
    pub const FRONT_RIGHT_CENTRE: Self = Self::from_bits(1 << 7);
    pub const REAR_CENTRE: Self = Self::from_bits(1 << 8);
    pub const SIDE_LEFT: Self = Self::from_bits(1 << 9);
    pub const SIDE_RIGHT: Self = Self::from_bits(1 << 10);
    pub const TOP_CENTRE: Self = Self::from_bits(1 << 11);
    pub const TOP_FRONT_LEFT: Self = Self::from_bits(1 << 12);
    pub const TOP_FRONT_CENTRE: Self = Self::from_bits(1 << 13);
    pub const TOP_FRONT_RIGHT: Self = Self::from_bits(1 << 14);
    pub const TOP_REAR_LEFT: Self = Self::from_bits(1 << 15);
    pub const TOP_REAR_CENTRE: Self = Self::from_bits(1 << 16);
    pub const TOP_REAR_RIGHT: Self = Self::from_bits(1 << 17);
    pub const REAR_LEFT_CENTRE: Self = Self::from_bits(1 << 18);
    pub const REAR_RIGHT_CENTRE: Self = Self::from_bits(1 << 19);
    pub const FRONT_LEFT_WIDE: Self = Self::from_bits(1 << 20);
    pub const FRONT_RIGHT_WIDE: Self = Self::from_bits(1 << 21);
    pub const FRONT_LEFT_HIGH: Self = Self::from_bits(1 << 22);
    pub const FRONT_CENTRE_HIGH: Self = Self::from_bits(1 << 23);
    pub const FRONT_RIGHT_HIGH: Self = Self::from_bits(1 << 24);
    pub const LFE2: Self = Self::from_bits(1 << 25);

    pub const MONO: Self = Self::FRONT_LEFT;
    pub const STEREO: Self = Self::from_bits(Self::FRONT_LEFT.mask | Self::FRONT_RIGHT.mask);
    pub const STEREO_2_1: Self = Self::from_bits(Self::STEREO.mask | Self::LFE1.mask);
    pub const SURROUND_5_1: Self = Self::from_bits(
        Self::STEREO_2_1.mask
            | Self::FRONT_CENTRE.mask
            | Self::REAR_LEFT.mask
            | Self::REAR_RIGHT.mask,
    );

    pub const SURROUND_7_1: Self =
        Self::from_bits(Self::SURROUND_5_1.mask | Self::SIDE_LEFT.mask | Self::SIDE_RIGHT.mask);

    #[inline]
    pub fn count(&self) -> u32 {
        self.mask.count_ones()
    }
}

impl BitAnd for ChannelLayout {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.mask & rhs.mask)
    }
}

impl BitOr for ChannelLayout {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.mask | rhs.mask)
    }
}

impl BitXor for ChannelLayout {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::from_bits(self.mask ^ rhs.mask)
    }
}

impl<S> SignalSpec<S> {
    pub fn builder() -> SignalSpecBuilder<S> {
        SignalSpecBuilder::new()
    }

    fn cast_sample_type<T>(self, sample_type: T) -> SignalSpec<T> {
        SignalSpec {
            sample_type,
            channels: self.channels,
            frame_rate: self.frame_rate,
            block_size: self.block_size,
            n_blocks: self.n_blocks,
        }
    }

    pub fn block_rate(&self) -> u32 {
        self.frame_rate / self.block_size as u32
    }

    pub fn samples_per_block(&self) -> usize {
        self.block_size * self.channels.count() as usize
    }

    pub fn n_frames(&self) -> Option<u64> {
        self.n_blocks.map(|n| n * self.block_size as u64)
    }

    pub fn n_samples(&self) -> Option<u64> {
        self.n_frames().map(|n| n * self.channels.count() as u64)
    }
}

macro_rules! impl_signal_spec {
    ($sample:ty, $dyn_sample:ident) => {
        impl From<SignalSpec<$sample>> for SignalSpec<SampleType> {
            fn from(spec: SignalSpec<$sample>) -> Self {
                Self {
                    sample_type: SampleType::$dyn_sample,
                    channels: spec.channels,
                    frame_rate: spec.frame_rate,
                    block_size: spec.block_size,
                    n_blocks: spec.n_blocks,
                }
            }
        }

        
    };
}

// impl<S: Sample> SignalSpec<S> {
//     pub fn to_dyn_sample(self) -> SignalSpec<SampleType> {
//         self.cast_sample_type(S::TYPE)
//     }
// }

// impl SignalSpec<SampleType> {
//     pub fn unwrap_sample_type<S: Sample>(self) -> Result<SignalSpec<S>, SyphonError> {
//         if self.sample_type != S::TYPE {
//             return Err(SyphonError::SignalMismatch);
//         }

//         Ok(self.cast_sample_type(S::ORIGIN))
//     }
// }

impl<S: Sample> TryFrom<SignalSpecBuilder<S>> for SignalSpec<S> {
    type Error = SyphonError;

    fn try_from(builder: SignalSpecBuilder<S>) -> Result<Self, Self::Error> {
        Ok(Self {
            sample_type: S::ORIGIN,
            channels: builder.channels.ok_or(SyphonError::InvalidData)?,
            frame_rate: builder.frame_rate.ok_or(SyphonError::InvalidData)?,
            block_size: builder.block_size.unwrap_or(1),
            n_blocks: builder.n_blocks,
        })
    }
}

impl TryFrom<SignalSpecBuilder<SampleType>> for SignalSpec<SampleType> {
    type Error = SyphonError;

    fn try_from(builder: SignalSpecBuilder<SampleType>) -> Result<Self, Self::Error> {
        Ok(Self {
            sample_type: builder.sample_type.ok_or(SyphonError::InvalidData)?,
            channels: builder.channels.ok_or(SyphonError::InvalidData)?,
            frame_rate: builder.frame_rate.ok_or(SyphonError::InvalidData)?,
            block_size: builder.block_size.unwrap_or(1),
            n_blocks: builder.n_blocks,
        })
    }
}

impl<S> SignalSpecBuilder<S> {
    pub fn new() -> Self {
        Self {
            sample_type: None,
            frame_rate: None,
            channels: None,
            block_size: None,
            n_blocks: None,
        }
    }

    pub fn samples_per_block(&self) -> Option<usize> {
        self.block_size
            .zip(self.channels)
            .map(|(block_size, channels)| block_size * channels.count() as usize)
    }

    pub fn n_frames(&self) -> Option<u64> {
        self.n_blocks
            .zip(self.block_size)
            .map(|(n_blocks, block_size)| n_blocks * block_size as u64)
    }

    pub fn n_samples(&self) -> Option<u64> {
        self.n_frames()
            .zip(self.channels)
            .map(|(n_frames, channels)| n_frames * channels.count() as u64)
    }

    pub fn is_empty(&self) -> bool {
        self.sample_type.is_none()
            && self.channels.is_none()
            && self.frame_rate.is_none()
            && self.block_size.is_none()
            && self.n_blocks.is_none()
    }

    pub fn frame_rate(mut self, frame_rate: u32) -> Self {
        self.frame_rate = Some(frame_rate);
        self
    }

    pub fn hz(mut self, hz: u32) -> Self {
        self.frame_rate = Some(hz);
        self
    }

    pub fn channels(mut self, channels: Channels) -> Self {
        self.channels = Some(channels);
        self
    }

    pub fn n_channels(mut self, n_channels: u32) -> Self {
        self.channels = Some(Channels::Count(n_channels));
        self
    }

    pub fn channel_layout(mut self, layout: ChannelLayout) -> Self {
        self.channels = Some(Channels::Layout(layout));
        self
    }

    pub fn mono(mut self) -> Self {
        self.channels = Some(ChannelLayout::MONO.into());
        self
    }

    pub fn stereo(mut self) -> Self {
        self.channels = Some(ChannelLayout::STEREO.into());
        self
    }

    pub fn block_size(mut self, block_size: usize) -> Self {
        self.block_size = Some(block_size);
        self
    }

    pub fn n_blocks(mut self, n_blocks: u64) -> Self {
        self.n_blocks = Some(n_blocks);
        self
    }

    pub fn build(self) -> Result<SignalSpec<S>, SyphonError>
    where
        Self: TryInto<SignalSpec<S>, Error = SyphonError>,
    {
        self.try_into()
    }
}

impl SignalSpecBuilder<SampleType> {
    pub fn sample_type(mut self, sample_type: SampleType) -> Self {
        self.sample_type = Some(sample_type);
        self
    }

    pub fn const_sample_type<S: Sample>(mut self) -> Self {
        self.sample_type = Some(S::TYPE);
        self
    }
}

impl<S> From<SignalSpec<S>> for SignalSpecBuilder<S> {
    fn from(spec: SignalSpec<S>) -> Self {
        Self {
            sample_type: Some(spec.sample_type),
            frame_rate: Some(spec.frame_rate),
            channels: Some(spec.channels),
            block_size: Some(spec.block_size),
            n_blocks: spec.n_blocks,
        }
    }
}

impl<S: Sample> From<SignalSpec<S>> for SignalSpecBuilder<SampleType> {
    fn from(spec: SignalSpec<S>) -> Self {
        Self {
            sample_type: Some(S::TYPE),
            frame_rate: Some(spec.frame_rate),
            channels: Some(spec.channels),
            block_size: Some(spec.block_size),
            n_blocks: spec.n_blocks,
        }
    }
}

pub trait Signal<S: Sample> {
    fn spec(&self) -> &SignalSpec<S>;

    fn adapt_sample_type<O: Sample>(self) -> SampleTypeAdapter<Self, S, O>
    where
        Self: Sized,
    {
        SampleTypeAdapter::new(self)
    }

    fn adapt_frame_rate(self, frame: u32) -> FrameRateAdapter<Self, S>
    where
        Self: Sized,
    {
        FrameRateAdapter::new(self, frame)
    }

    fn adapt_channels(self, channels: Channels) -> ChannelsAdapter<Self, S>
    where
        Self: Sized,
    {
        ChannelsAdapter::new(self, channels)
    }

    fn adapt_block_size(self, block_size: usize) -> BlockSizeAdapter<Self, S>
    where
        Self: Sized,
    {
        BlockSizeAdapter::new(self, block_size)
    }

    fn adapt_n_blocks(self, n_blocks: u64) -> NBlocksAdapter<Self, S>
    where
        Self: Sized,
    {
        NBlocksAdapter::new(self, n_blocks)
    }

    fn adapt_seconds(self, seconds: f64) -> NBlocksAdapter<Self, S>
    where
        Self: Sized,
    {
        NBlocksAdapter::from_seconds(self, seconds)
    }

    fn adapt_duration(self, duration: Duration) -> NBlocksAdapter<Self, S>
    where
        Self: Sized,
    {
        NBlocksAdapter::from_duration(self, duration)
    }

    fn chain<T: Signal<S>>(self, other: T) -> SignalChain<Self, T, S>
    where
        Self: Sized,
    {
        SignalChain::new(self, other)
    }
}

pub trait SignalReader<S: Sample>: Signal<S> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError>;

    fn read_exact(&mut self, buffer: &mut [S]) -> Result<(), SyphonError> {
        let buf_len = buffer.len();
        if buf_len % self.spec().samples_per_block() != 0 {
            return Err(SyphonError::SignalMismatch);
        }

        let mut n_read: usize = 0;
        while n_read < buf_len {
            n_read += match self.read(&mut buffer[n_read..]) {
                Ok(0) => break,
                Ok(n) => n * self.spec().samples_per_block(),
                Err(SyphonError::Interrupted) => continue,
                Err(e) => return Err(e),
            }
        }

        if n_read != buf_len {
            return Err(SyphonError::EndOfStream);
        }

        Ok(())
    }
}

pub trait SignalWriter<S: Sample>: Signal<S> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError>;

    fn write_exact(&mut self, buffer: &[S]) -> Result<(), SyphonError> {
        let buf_len = buffer.len();
        let samples_per_block = self.spec().samples_per_block();
        if buf_len % samples_per_block != 0 {
            return Err(SyphonError::SignalMismatch);
        }

        let mut n_written: usize = 0;
        while n_written < buf_len {
            n_written += match self.write(&buffer[n_written..]) {
                Ok(0) => break,
                Ok(n) => n * samples_per_block,
                Err(SyphonError::Interrupted) => continue,
                Err(e) => return Err(e),
            }
        }

        if n_written != buf_len {
            return Err(SyphonError::EndOfStream);
        }

        Ok(())
    }
}

impl<S: Sample> Signal<S> for Box<dyn SignalReader<S>> {
    fn spec(&self) -> &SignalSpec<S> {
        self.as_ref().spec()
    }
}

impl<S: Sample> SignalReader<S> for Box<dyn SignalReader<S>> {
    fn read(&mut self, buffer: &mut [S]) -> Result<usize, SyphonError> {
        self.as_mut().read(buffer)
    }
}

impl<S: Sample> Signal<S> for Box<dyn SignalWriter<S>> {
    fn spec(&self) -> &SignalSpec<S> {
        self.as_ref().spec()
    }
}

impl<S: Sample> SignalWriter<S> for Box<dyn SignalWriter<S>> {
    fn write(&mut self, buffer: &[S]) -> Result<usize, SyphonError> {
        self.as_mut().write(buffer)
    }
}

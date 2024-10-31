use phonic_core::PhonicError;
use std::ops::{BitAnd, BitOr, BitXor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channels {
    Count(u32),
    Layout(ChannelLayout),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChannelLayout {
    mask: u32,
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

    pub fn is_compatible(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Layout(a), Self::Layout(b)) => a == b,
            (a, b) => a.count() == b.count(),
        }
    }

    pub fn merge(&mut self, other: &Self) -> Result<(), PhonicError> {
        match (&*self, other) {
            (Self::Layout(a), Self::Layout(b)) if a == b => Ok(()),
            (Self::Count(a), Self::Count(b)) if a == b => Ok(()),

            (Self::Layout(layout), Self::Count(count))
            | (Self::Count(count), Self::Layout(layout))
                if layout.count() == *count =>
            {
                *self = Self::Layout(*layout);
                Ok(())
            }

            _ => Err(PhonicError::SignalMismatch),
        }
    }

    pub fn merged(mut self, other: &Self) -> Result<Self, PhonicError> {
        self.merge(other)?;
        Ok(self)
    }
}

impl From<u32> for Channels {
    fn from(n: u32) -> Self {
        Self::Count(n)
    }
}

impl From<ChannelLayout> for Channels {
    fn from(layout: ChannelLayout) -> Self {
        Self::Layout(layout)
    }
}

impl ChannelLayout {
    pub const fn from_mask(mask: u32) -> Self {
        Self { mask }
    }

    pub fn count(&self) -> u32 {
        self.mask.count_ones()
    }

    pub const FRONT_LEFT: Self = Self::from_mask(1 << 0);
    pub const FRONT_RIGHT: Self = Self::from_mask(1 << 1);
    pub const FRONT_CENTRE: Self = Self::from_mask(1 << 2);
    pub const LFE1: Self = Self::from_mask(1 << 3);
    pub const REAR_LEFT: Self = Self::from_mask(1 << 4);
    pub const REAR_RIGHT: Self = Self::from_mask(1 << 5);
    pub const FRONT_LEFT_CENTRE: Self = Self::from_mask(1 << 6);
    pub const FRONT_RIGHT_CENTRE: Self = Self::from_mask(1 << 7);
    pub const REAR_CENTRE: Self = Self::from_mask(1 << 8);
    pub const SIDE_LEFT: Self = Self::from_mask(1 << 9);
    pub const SIDE_RIGHT: Self = Self::from_mask(1 << 10);
    pub const TOP_CENTRE: Self = Self::from_mask(1 << 11);
    pub const TOP_FRONT_LEFT: Self = Self::from_mask(1 << 12);
    pub const TOP_FRONT_CENTRE: Self = Self::from_mask(1 << 13);
    pub const TOP_FRONT_RIGHT: Self = Self::from_mask(1 << 14);
    pub const TOP_REAR_LEFT: Self = Self::from_mask(1 << 15);
    pub const TOP_REAR_CENTRE: Self = Self::from_mask(1 << 16);
    pub const TOP_REAR_RIGHT: Self = Self::from_mask(1 << 17);
    pub const REAR_LEFT_CENTRE: Self = Self::from_mask(1 << 18);
    pub const REAR_RIGHT_CENTRE: Self = Self::from_mask(1 << 19);
    pub const FRONT_LEFT_WIDE: Self = Self::from_mask(1 << 20);
    pub const FRONT_RIGHT_WIDE: Self = Self::from_mask(1 << 21);
    pub const FRONT_LEFT_HIGH: Self = Self::from_mask(1 << 22);
    pub const FRONT_CENTRE_HIGH: Self = Self::from_mask(1 << 23);
    pub const FRONT_RIGHT_HIGH: Self = Self::from_mask(1 << 24);
    pub const LFE2: Self = Self::from_mask(1 << 25);

    pub const MONO: Self = Self::FRONT_CENTRE;
    pub const STEREO: Self = Self::from_mask(Self::FRONT_LEFT.mask | Self::FRONT_RIGHT.mask);

    pub const STEREO_2_1: Self = Self::from_mask(Self::STEREO.mask | Self::LFE1.mask);

    pub const SURROUND_5_1: Self = Self::from_mask(
        Self::STEREO_2_1.mask
            | Self::FRONT_CENTRE.mask
            | Self::REAR_LEFT.mask
            | Self::REAR_RIGHT.mask,
    );

    pub const SURROUND_7_1: Self =
        Self::from_mask(Self::SURROUND_5_1.mask | Self::SIDE_LEFT.mask | Self::SIDE_RIGHT.mask);
}

impl BitAnd for ChannelLayout {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_mask(self.mask & rhs.mask)
    }
}

impl BitOr for ChannelLayout {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_mask(self.mask | rhs.mask)
    }
}

impl BitXor for ChannelLayout {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::from_mask(self.mask ^ rhs.mask)
    }
}

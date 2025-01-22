use crate::codecs::pcm::PcmCodecTag;

pub(super) enum Endianess {
    Little,
    Big,
}

impl Endianess {
    pub fn is_native(&self) -> bool {
        match self {
            Self::Little if cfg!(target_endian = "little") => true,
            Self::Big if cfg!(target_endian = "big") => true,
            _ => false,
        }
    }
}

impl From<PcmCodecTag> for Endianess {
    fn from(tag: PcmCodecTag) -> Self {
        match tag {
            PcmCodecTag::LE => Self::Little,
            PcmCodecTag::BE => Self::Big,
        }
    }
}

impl From<Endianess> for PcmCodecTag {
    fn from(value: Endianess) -> Self {
        match value {
            Endianess::Little => PcmCodecTag::LE,
            Endianess::Big => PcmCodecTag::BE,
        }
    }
}

impl Default for Endianess {
    fn default() -> Self {
        if cfg!(target_endian = "little") {
            Self::Little
        } else if cfg!(target_endian = "big") {
            Self::Big
        } else {
            panic!("unknown target endianess")
        }
    }
}

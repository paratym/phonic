use crate::KnownCodec;
use lazy_static::lazy_static;
use std::collections::HashMap;
use syphon_core::SyphonError;
use syphon_format_wave::WAVE_IDENTIFIERS;
use syphon_io_core::{
    utils::{FormatIdentifier, FormatIdentifiers},
    DynFormat, DynFormatConstructor, FormatData, FormatTag, StdIoStream,
};

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
#[non_exhaustive]
pub enum KnownFormat {
    #[cfg(feature = "wave")]
    Wave,
}

lazy_static! {
    static ref KNOWN_FORMAT_IDENTIFIERS: HashMap<KnownFormat, &'static FormatIdentifiers> = {
        let mut map = HashMap::new();

        #[cfg(feature = "wave")]
        map.insert(KnownFormat::Wave, &WAVE_IDENTIFIERS);

        map
    };
}

impl FormatTag for KnownFormat {
    type Codec = KnownCodec;

    fn fill_data(data: &mut FormatData<Self>) -> Result<(), SyphonError> {
        match data.format {
            #[cfg(feature = "wave")]
            Some(Self::Wave) => crate::formats::wave::fill_wave_data(data),

            _ => return Ok(()),
        }
    }
}

impl DynFormatConstructor for KnownFormat {
    type Tag = Self;

    fn from_std_io<S: StdIoStream + 'static>(
        &self,
        inner: S,
    ) -> Result<Box<dyn DynFormat<Tag = Self::Tag>>, SyphonError> {
        Ok(match self {
            #[cfg(feature = "wave")]
            KnownFormat::Wave => Box::new(crate::formats::wave::WaveFormat::new(inner)?),

            _ => return Err(SyphonError::Unsupported),
        })
    }
}

impl<'a> TryFrom<&FormatIdentifier<'a>> for KnownFormat {
    type Error = SyphonError;

    fn try_from(id: &FormatIdentifier<'a>) -> Result<Self, Self::Error> {
        KNOWN_FORMAT_IDENTIFIERS
            .iter()
            .find(|(_, ids)| ids.contains(id))
            .map(|(fmt, _)| *fmt)
            .ok_or(SyphonError::NotFound)
    }
}

#[cfg(feature = "wave")]
impl From<crate::formats::wave::WaveFormatTag> for KnownFormat {
    fn from(_: crate::formats::wave::WaveFormatTag) -> Self {
        Self::Wave
    }
}

#[cfg(feature = "wave")]
impl TryFrom<KnownFormat> for crate::formats::wave::WaveFormatTag {
    type Error = SyphonError;

    fn try_from(format: KnownFormat) -> Result<Self, Self::Error> {
        match format {
            KnownFormat::Wave => Ok(Self),
            _ => Err(SyphonError::Unsupported),
        }
    }
}

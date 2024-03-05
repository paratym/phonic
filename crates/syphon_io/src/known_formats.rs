use crate::KnownCodec;
use std::{
    hash::Hash,
    io::{Read, Write},
};
use syphon_core::SyphonError;
use syphon_io_core::{
    utils::{FormatIdentifier, FormatIdentifiers},
    FormatData, FormatReader, FormatRegistry, FormatTag, FormatWriter,
};

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
#[non_exhaustive]
pub enum KnownFormat {
    Wave,
}

impl KnownFormat {
    pub fn all() -> &'static [Self] {
        &[
            #[cfg(feature = "wave")]
            KnownFormat::Wave,
        ]
    }

    pub fn identifiers(&self) -> Option<&'static FormatIdentifiers> {
        Some(match self {
            #[cfg(feature = "wave")]
            &Self::Wave => &crate::formats::wave::WAVE_IDENTIFIERS,

            _ => return None,
        })
    }
}

impl FormatTag for KnownFormat {
    type Codec = KnownCodec;
}

impl FormatRegistry for KnownFormat {
    fn fill_data(data: &mut FormatData<Self>) -> Result<(), SyphonError> {
        match data.format {
            #[cfg(feature = "wave")]
            Some(Self::Wave) => crate::formats::wave::fill_wave_format_data(data),

            _ => Ok(()),
        }
    }

    fn demux_reader(
        &self,
        inner: impl Read + 'static,
    ) -> Result<Box<dyn FormatReader<Tag = Self>>, SyphonError> {
        Ok(match self {
            #[cfg(feature = "wave")]
            KnownFormat::Wave => Box::new(crate::formats::wave::WaveFormat::new(inner)?),

            _ => return Err(SyphonError::Unsupported),
        })
    }

    fn mux_writer(
        &self,
        inner: impl Write + 'static,
    ) -> Result<Box<dyn FormatWriter<Tag = Self>>, SyphonError> {
        Ok(match self {
            #[cfg(feature = "wave")]
            KnownFormat::Wave => Box::new(crate::formats::wave::WaveFormat::new(inner)?),

            _ => return Err(SyphonError::Unsupported),
        })
    }

    fn mux_reader(
        inner: impl FormatReader<Tag = Self> + 'static,
    ) -> Result<Box<dyn Read>, SyphonError> {
        todo!()
    }

    fn demux_writer(
        inner: impl FormatWriter<Tag = Self> + 'static,
    ) -> Result<Box<dyn Write>, SyphonError> {
        todo!()
    }
}

impl<'a> TryFrom<&FormatIdentifier<'a>> for KnownFormat {
    type Error = SyphonError;

    fn try_from(id: &FormatIdentifier<'a>) -> Result<Self, Self::Error> {
        Self::all()
            .iter()
            .find(|fmt| fmt.identifiers().is_some_and(|ids| ids.contains(id)))
            .copied()
            .ok_or(SyphonError::Unsupported)
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
            KnownFormat::Wave => Ok(Self()),
            _ => Err(SyphonError::Unsupported),
        }
    }
}

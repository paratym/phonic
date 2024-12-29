use crate::KnownCodec;
use lazy_static::lazy_static;
use phonic_io_core::{
    utils::{DropFinalize, FormatIdentifier, PollIo},
    DynFormat, DynFormatConstructor, FormatConstructor, FormatTag, StdIoSource, StreamSpec,
};
use phonic_signal::{PhonicError, PhonicResult};
use std::collections::HashMap;

lazy_static! {
    static ref KNOWN_FILE_EXTENSIONS: HashMap<&'static str, KnownFormat> = {
        use crate::formats::*;
        let mut map = HashMap::new();

        #[cfg(feature = "wave")]
        map.extend(
            wave::WAVE_IDENTIFIERS
                .file_extensions
                .iter()
                .map(|ext| (*ext, KnownFormat::Wave)),
        );

        map
    };
    static ref KNOWN_MIME_TYPES: HashMap<&'static str, KnownFormat> = {
        use crate::formats::*;
        let mut map = HashMap::new();

        #[cfg(feature = "wave")]
        map.extend(
            wave::WAVE_IDENTIFIERS
                .mime_types
                .iter()
                .map(|mime| (*mime, KnownFormat::Wave)),
        );

        map
    };
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
#[non_exhaustive]
pub enum KnownFormat {
    #[cfg(feature = "wave")]
    Wave,
}

impl FormatTag for KnownFormat {
    type Codec = KnownCodec;
}

impl DynFormatConstructor for KnownFormat {
    fn read_index<T>(&self, inner: T) -> PhonicResult<Box<dyn DynFormat<Tag = Self>>>
    where
        T: StdIoSource + 'static,
    {
        use crate::formats::*;

        Ok(match self {
            #[cfg(feature = "wave")]
            Self::Wave => Box::new(DropFinalize(PollIo(wave::WaveFormat::read_index(inner)?))),

            #[allow(unreachable_patterns)]
            _ => return Err(PhonicError::Unsupported),
        })
    }

    fn write_index<T, I>(&self, inner: T, index: I) -> PhonicResult<Box<dyn DynFormat<Tag = Self>>>
    where
        T: StdIoSource + 'static,
        I: IntoIterator<Item = StreamSpec<Self::Codec>>,
    {
        use crate::formats::*;

        Ok(match self {
            #[cfg(feature = "wave")]
            Self::Wave => Box::new(DropFinalize(PollIo(wave::WaveFormat::write_index(
                inner, index,
            )?))),

            #[allow(unreachable_patterns)]
            _ => return Err(PhonicError::Unsupported),
        })
    }
}

impl<'a> TryFrom<FormatIdentifier<'a>> for KnownFormat {
    type Error = PhonicError;

    fn try_from(id: FormatIdentifier<'a>) -> Result<Self, Self::Error> {
        let format = match id {
            FormatIdentifier::FileExtension(ext) => KNOWN_FILE_EXTENSIONS.get(ext),
            FormatIdentifier::MimeType(mime) => KNOWN_MIME_TYPES.get(mime),
        };

        format.copied().ok_or(PhonicError::Unsupported)
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
    type Error = PhonicError;

    fn try_from(format: KnownFormat) -> Result<Self, Self::Error> {
        match format {
            KnownFormat::Wave => Ok(Self),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

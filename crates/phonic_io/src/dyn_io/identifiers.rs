use crate::dyn_io::KnownFormat;
use phonic_signal::PhonicError;
use std::{collections::HashMap, ffi::OsStr, path::Path, sync::LazyLock};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FormatIdentifier<'a> {
    FileExtension(&'a str),
    MimeType(&'a str),
}

pub static KNOWN_FILE_EXT: LazyLock<HashMap<&'static str, KnownFormat>> = LazyLock::new(|| {
    use crate::formats::*;

    let mut map = HashMap::new();

    #[cfg(feature = "wave")]
    map.extend(
        wave::KNOWN_WAVE_FILE_EXTENSIONS
            .into_iter()
            .map(|ext| (ext, KnownFormat::Wave)),
    );

    map
});

pub static KNOWN_MIME_TYPE: LazyLock<HashMap<&'static str, KnownFormat>> = LazyLock::new(|| {
    use crate::formats::*;
    let mut map = HashMap::new();

    #[cfg(feature = "wave")]
    map.extend(
        wave::KNOWN_WAVE_MIME_TYPES
            .into_iter()
            .map(|ext| (ext, KnownFormat::Wave)),
    );

    map
});

impl FormatIdentifier<'_> {
    pub fn known_format(self) -> Option<KnownFormat> {
        let entry = match self {
            Self::FileExtension(ext) => KNOWN_FILE_EXT.get(ext),
            Self::MimeType(mime) => KNOWN_MIME_TYPE.get(mime),
        };

        entry.copied()
    }
}

impl<'a> TryFrom<&'a Path> for FormatIdentifier<'a> {
    type Error = PhonicError;

    fn try_from(path: &'a Path) -> Result<Self, Self::Error> {
        path.extension()
            .and_then(OsStr::to_str)
            .map(FormatIdentifier::FileExtension)
            .ok_or(PhonicError::MissingData)
    }
}

impl<'a> TryFrom<FormatIdentifier<'a>> for KnownFormat {
    type Error = PhonicError;

    fn try_from(id: FormatIdentifier<'a>) -> Result<Self, Self::Error> {
        id.known_format().ok_or(PhonicError::NotFound)
    }
}

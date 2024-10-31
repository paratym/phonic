use phonic_core::PhonicError;
use std::{ffi::OsStr, path::Path};

pub struct FormatIdentifiers {
    pub file_extensions: &'static [&'static str],
    pub mime_types: &'static [&'static str],
    pub markers: &'static [&'static [u8]],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FormatIdentifier<'a> {
    FileExtension(&'a str),
    MimeType(&'a str),
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

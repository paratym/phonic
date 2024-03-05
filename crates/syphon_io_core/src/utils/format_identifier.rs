use std::path::Path;
use syphon_core::SyphonError;

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

impl FormatIdentifiers {
    pub fn contains(&self, identifier: &FormatIdentifier) -> bool {
        match identifier {
            FormatIdentifier::FileExtension(ext) => self.file_extensions.contains(ext),
            FormatIdentifier::MimeType(mime) => self.mime_types.contains(mime),
        }
    }
}

impl<'a> TryFrom<&'a Path> for FormatIdentifier<'a> {
    type Error = SyphonError;

    fn try_from(path: &'a Path) -> Result<Self, Self::Error> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| FormatIdentifier::FileExtension(ext))
            .ok_or(SyphonError::MissingData)
    }
}

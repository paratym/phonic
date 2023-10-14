use crate::{
    core::SyphonError,
    io::{
        codec_registry::SyphonCodec,
        formats::{WavReader, WAV_FORMAT_IDENTIFIERS},
        BufReader, FormatReadResult, FormatReader, MediaSource, TrackDataBuilder,
        UnseekableMediaSource,
    },
};
use std::{
    collections::HashMap,
    hash::Hash,
    io::{Read, Seek, SeekFrom},
    marker::PhantomData,
};

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub enum SyphonFormat {
    Wav,
    Other(&'static str),
}

pub struct FormatIdentifiers {
    pub file_extensions: &'static [&'static str],
    pub mime_types: &'static [&'static str],
    pub markers: &'static [&'static [u8]],
}

pub enum FormatIdentifier<'a> {
    FileExtension(&'a str),
    MimeType(&'a str),
}

const MAX_MARKER_INDEX: usize = 1024;

impl FormatIdentifiers {
    fn contains(&self, identifier: &FormatIdentifier) -> bool {
        match identifier {
            FormatIdentifier::FileExtension(ext) => self.file_extensions.contains(ext),
            FormatIdentifier::MimeType(mime) => self.mime_types.contains(mime),
        }
    }
}

pub struct FormatRegistry<K, C> {
    formats: HashMap<
        K,
        (
            Option<&'static FormatIdentifiers>,
            Option<Box<dyn Fn(Box<dyn MediaSource>) -> Box<dyn FormatReader<CodecKey = C>>>>,
        ),
    >,
}

impl<K: Hash + Eq + Copy, C> FormatRegistry<K, C> {
    pub fn new() -> Self {
        Self {
            formats: HashMap::new(),
        }
    }

    pub fn register_identifiers(mut self, key: K, identifiers: &'static FormatIdentifiers) -> Self {
        let entry = self.formats.entry(key).or_insert((None, None));
        entry.0 = Some(identifiers);

        self
    }

    pub fn register_reader(
        mut self,
        key: K,
        reader_constructor: impl Fn(Box<dyn MediaSource>) -> Box<dyn FormatReader<CodecKey = C>>
            + 'static,
    ) -> Self {
        let format = self.formats.entry(key).or_insert((None, None));
        format.1 = Some(Box::new(reader_constructor));

        self
    }

    pub fn register_format(
        mut self,
        key: K,
        identifiers: &'static FormatIdentifiers,
        reader_constructor: impl Fn(Box<dyn MediaSource>) -> Box<dyn FormatReader<CodecKey = C>>
            + 'static,
    ) -> Self {
        self.formats
            .insert(key, (Some(identifiers), Some(Box::new(reader_constructor))));

        self
    }

    pub fn construct_reader(
        &self,
        key: &K,
        source: impl MediaSource + 'static,
    ) -> Option<Box<dyn FormatReader<CodecKey = C>>> {
        Some(self.formats.get(key)?.1.as_ref()?(Box::new(source)))
    }

    pub fn resolve_format(
        &self,
        reader: &mut (impl MediaSource + 'static),
        identifier: Option<FormatIdentifier>,
    ) -> Option<K> {
        // self.formats
        //     .iter()
        //     .filter_map(|(key, format)| Some((key, format.0.as_ref()?)))
        //     .find_map(|(k, identifiers)| {
        //         if let Some(hint) = hint {
        //             if hint.matches(identifiers) {
        //                 return Some(*k);
        //             }
        //         }

        //         // TODO: check for markers

        //         None
        //     })

        None
    }

    pub fn resolve_reader(
        &self,
        mut reader: impl MediaSource + 'static,
        identifier: Option<FormatIdentifier>,
    ) -> Option<Box<dyn FormatReader<CodecKey = C>>> {
        let key = self.resolve_format(&mut reader, identifier)?;
        self.construct_reader(&key, Box::new(reader))
    }
}

pub struct CodecKeyConverter<T: FormatReader, K: TryFrom<T::CodecKey>> {
    reader: T,
    _key: PhantomData<K>,
}

impl<T: FormatReader, K: TryFrom<T::CodecKey>> CodecKeyConverter<T, K> {
    pub fn new(reader: T) -> Self {
        Self {
            reader,
            _key: PhantomData,
        }
    }
}

impl<T: FormatReader, K: Copy + TryFrom<T::CodecKey>> FormatReader for CodecKeyConverter<T, K> {
    type CodecKey = K;

    fn tracks(&self) -> Box<dyn Iterator<Item = TrackDataBuilder<Self::CodecKey>>> {
        todo!()
    }

    fn read_headers(&mut self) -> Result<usize, SyphonError> {
        self.reader.read_headers()
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<FormatReadResult, SyphonError> {
        self.reader.read(buf)
    }

    fn seek(&mut self, offset: SeekFrom) -> Result<usize, SyphonError> {
        self.reader.seek(offset)
    }
}

pub fn syphon_format_registry() -> FormatRegistry<SyphonFormat, SyphonCodec> {
    FormatRegistry::new().register_format(SyphonFormat::Wav, &WAV_FORMAT_IDENTIFIERS, |reader| {
        Box::new(CodecKeyConverter::new(WavReader::new(reader)))
    })
}

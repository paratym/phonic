use crate::{
    io::{
        EncodedStreamReader, EncodedStreamSpec, EncodedStreamSpecBuilder, EncodedStreamWriter,
        FormatReader, FormatWriter,
    },
    SyphonError,
};
use std::io::{self, Read, Seek, SeekFrom, Write};

pub struct Track<T> {
    inner: T,
    stream_spec: EncodedStreamSpec,
    track_i: usize,
}

impl<T> Track<T> {
    fn default_i<'a>(
        tracks: impl Iterator<Item = &'a EncodedStreamSpecBuilder>,
    ) -> Result<usize, SyphonError> {
        Ok(tracks
            .enumerate()
            .find(|(_, spec)| spec.codec_key.is_some() && spec.decoded_spec.sample_format.is_some())
            .ok_or(SyphonError::Empty)?
            .0)
    }

    pub fn reader(inner: T, track_i: usize) -> Result<Self, SyphonError>
    where
        T: FormatReader,
    {
        let stream_spec = inner
            .format_data()
            .tracks
            .iter()
            .nth(track_i)
            .ok_or(SyphonError::BadRequest)?
            .try_build()?;

        Ok(Self {
            inner,
            stream_spec,
            track_i,
        })
    }

    pub fn default_reader(inner: T) -> Result<Self, SyphonError>
    where
        T: FormatReader,
    {
        let track_i = Self::default_i(inner.format_data().tracks.iter())?;
        Self::reader(inner, track_i)
    }

    pub fn writer(inner: T, track_i: usize) -> Result<Self, SyphonError>
    where
        T: FormatWriter,
    {
        let stream_spec = inner
            .format_data()
            .tracks
            .iter()
            .nth(track_i)
            .ok_or(SyphonError::BadRequest)?
            .try_build()?;

        Ok(Self {
            inner,
            stream_spec,
            track_i,
        })
    }

    pub fn default_writer(inner: T) -> Result<Self, SyphonError>
    where
        T: FormatWriter,
    {
        let track_i = Self::default_i(inner.format_data().tracks.iter())?;
        Self::writer(inner, track_i)
    }
}

impl<T: FormatReader> Read for Track<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let result = self.inner.read(buf)?;
            if result.track_i == self.track_i {
                return Ok(result.n);
            }
        }
    }
}

impl<T: FormatWriter> Write for Track<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(self.inner.write(self.track_i, buf)?)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(self.inner.flush()?)
    }
}

impl<T: Seek> Seek for Track<T> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        Ok(self.inner.seek(offset)?)
    }
}

impl<R: FormatReader> EncodedStreamReader for Track<R> {
    fn stream_spec(&self) -> &EncodedStreamSpec {
        &self.stream_spec
    }
}

impl<W: FormatWriter> EncodedStreamWriter for Track<W> {
    fn stream_spec(&self) -> &EncodedStreamSpec {
        &self.stream_spec
    }
}

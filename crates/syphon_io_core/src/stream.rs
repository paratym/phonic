use crate::{CodecRegistry, CodecTag};
use std::any::TypeId;
use syphon_core::SyphonError;
use syphon_signal::{Sample, Signal, SignalSpecBuilder, TaggedSignalReader, TaggedSignalWriter};

#[derive(Debug, Clone, Copy)]
pub struct StreamSpec<C: CodecTag> {
    pub codec: Option<C>,
    pub avg_bitrate: Option<f64>,
    pub block_align: Option<u16>,
    pub sample_type: Option<TypeId>,
    pub decoded_spec: SignalSpecBuilder,
}

impl<C: CodecTag> StreamSpec<C> {
    pub fn new() -> Self {
        Self {
            codec: None,
            avg_bitrate: None,
            block_align: None,
            sample_type: None,
            decoded_spec: SignalSpecBuilder::new(),
        }
    }

    pub fn with_tag_type<T>(self) -> StreamSpec<T>
    where
        T: CodecTag,
        C: TryInto<T>,
    {
        StreamSpec {
            codec: self.codec.and_then(|c| c.try_into().ok()),
            avg_bitrate: self.avg_bitrate,
            block_align: self.block_align,
            sample_type: self.sample_type,
            decoded_spec: self.decoded_spec,
        }
    }

    pub fn with_codec(mut self, codec: C) -> Self {
        self.codec = Some(codec);
        self
    }

    pub fn with_avg_bitrate(mut self, bitrate: f64) -> Self {
        self.avg_bitrate = Some(bitrate);
        self
    }

    pub fn with_block_align(mut self, block_align: u16) -> Self {
        self.block_align = Some(block_align);
        self
    }

    pub fn with_sample_type_id(mut self, sample_type: TypeId) -> Self {
        self.sample_type = Some(sample_type);
        self
    }

    pub fn with_sample_type<T: Sample + 'static>(mut self) -> Self {
        self.sample_type = Some(TypeId::of::<T>());
        self
    }

    pub fn with_decoded_spec(mut self, decoded_spec: SignalSpecBuilder) -> Self {
        self.decoded_spec = decoded_spec;
        self
    }

    pub fn n_bytes(&self) -> Option<u64> {
        self.avg_bitrate
            .zip(self.decoded_spec.duration())
            .map(|(r, d)| (r / 8.0 * d.as_secs_f64()) as u64)
    }

    pub fn is_empty(&self) -> bool {
        self.avg_bitrate.is_none()
            && self.block_align.is_none()
            && self.sample_type.is_none()
            && self.decoded_spec.is_empty()
    }

    pub fn merge(&mut self, other: Self) -> Result<(), SyphonError> {
        if let Some(codec) = other.codec {
            if self.codec.get_or_insert(codec) != &codec {
                return Err(SyphonError::SignalMismatch);
            }
        }

        if let Some(avg_bitrate) = other.avg_bitrate {
            if self.avg_bitrate.get_or_insert(avg_bitrate) != &avg_bitrate {
                return Err(SyphonError::SignalMismatch);
            }
        }

        if let Some(block_align) = other.block_align {
            if self
                .block_align
                .is_some_and(|align| align % block_align != 0)
            {
                return Err(SyphonError::SignalMismatch);
            }

            self.block_align = Some(block_align);
        }

        self.decoded_spec.merge(other.decoded_spec)
    }

    pub fn fill(&mut self) -> Result<(), SyphonError>
    where
        C: CodecRegistry,
    {
        C::fill_spec(self)
    }

    pub fn filled(mut self) -> Result<Self, SyphonError>
    where
        C: CodecRegistry,
    {
        self.fill()?;
        Ok(self)
    }
}

impl<T, C> From<&T> for StreamSpec<C>
where
    T: Signal,
    T::Sample: 'static,
    C: CodecTag,
{
    fn from(inner: &T) -> Self {
        Self {
            codec: None,
            avg_bitrate: None,
            block_align: None,
            sample_type: Some(TypeId::of::<T::Sample>()),
            decoded_spec: inner.spec().clone().into(),
        }
    }
}

pub trait Stream {
    type Tag: CodecTag;

    fn spec(&self) -> &StreamSpec<Self::Tag>;
}

pub trait StreamObserver: Stream {
    fn position(&self) -> Result<u64, SyphonError>;
}

pub trait StreamReader: Stream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, SyphonError>;

    fn into_decoder(self) -> Result<TaggedSignalReader, SyphonError>
    where
        Self: Sized + 'static,
        Self::Tag: CodecRegistry,
    {
        Self::Tag::decoder_reader(self)
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> Result<(), SyphonError> {
        if self
            .spec()
            .block_align
            .is_some_and(|a| buf.len() % a as usize != 0)
        {
            return Err(SyphonError::SignalMismatch);
        }

        while !buf.is_empty() {
            match self.read(&mut buf) {
                Ok(0) if buf.len() == 0 => break,
                Ok(0) => return Err(SyphonError::EndOfStream),
                Ok(n) => buf = &mut buf[n..],
                Err(SyphonError::Interrupted) => continue,
                Err(e) => return Err(e),
            };
        }

        Ok(())
    }
}

pub trait StreamWriter: Stream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, SyphonError>;
    fn flush(&mut self) -> Result<(), SyphonError>;

    fn into_encoder(self) -> Result<TaggedSignalWriter, SyphonError>
    where
        Self: Sized + 'static,
        Self::Tag: CodecRegistry,
    {
        Self::Tag::encoder_writer(self)
    }

    fn write_exact(&mut self, mut buf: &[u8]) -> Result<(), SyphonError> {
        if self
            .spec()
            .block_align
            .is_some_and(|a| buf.len() % a as usize != 0)
        {
            return Err(SyphonError::SignalMismatch);
        }

        while !buf.is_empty() {
            match self.write(&buf) {
                Ok(0) if buf.len() == 0 => break,
                Ok(0) => return Err(SyphonError::EndOfStream),
                Ok(n) => buf = &buf[n..],
                Err(SyphonError::Interrupted) => continue,
                Err(e) => return Err(e),
            };
        }

        Ok(())
    }

    fn write_all_buffered<R>(&mut self, reader: &mut R, buf: &mut [u8]) -> Result<u64, SyphonError>
    where
        Self: Sized,
        R: StreamReader,
        Self::Tag: TryInto<R::Tag>,
    {
        if let Err(e) = reader.spec().clone().merge(self.spec().with_tag_type()) {
            return Err(e);
        }

        let mut n_read = 0;
        loop {
            let n = match reader.read(buf) {
                Ok(0) | Err(SyphonError::EndOfStream) => return Ok(n_read),
                Ok(n) => n,
                Err(SyphonError::Interrupted) | Err(SyphonError::NotReady) => continue,
                Err(e) => return Err(e),
            };

            self.write_exact(&mut buf[..n])?;
            n_read += n as u64;
        }
    }

    fn write_all<R>(&mut self, reader: &mut R) -> Result<u64, SyphonError>
    where
        Self: Sized,
        R: StreamReader,
        Self::Tag: TryInto<R::Tag>,
    {
        let mut buffer = [0u8; 4096];
        self.write_all_buffered(reader, &mut buffer)
    }
}

pub trait StreamSeeker: Stream {
    fn seek(&mut self, offset: i64) -> Result<(), SyphonError>;

    fn set_position(&mut self, position: u64) -> Result<(), SyphonError>
    where
        Self: StreamObserver,
    {
        self.seek(self.position()? as i64 - position as i64)
    }
}

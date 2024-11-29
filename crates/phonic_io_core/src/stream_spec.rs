use crate::CodecTag;
use phonic_signal::{PhonicError, PhonicResult, Sample, Signal, SignalSpec, SignalSpecBuilder};
use std::{any::TypeId, fmt::Debug, time::Duration};

#[derive(Debug, Clone, Copy)]
pub struct StreamSpec<C: CodecTag> {
    pub codec: C,
    pub avg_byte_rate: u32,
    pub block_align: usize,
    pub sample_type: TypeId,
    pub decoded_spec: SignalSpec,
}

#[derive(Debug, Clone, Copy)]
pub struct StreamSpecBuilder<C: CodecTag> {
    pub codec: Option<C>,
    pub avg_byte_rate: Option<u32>,
    pub block_align: Option<usize>,
    pub sample_type: Option<TypeId>,
    pub decoded_spec: SignalSpecBuilder,
}

impl<C: CodecTag> StreamSpec<C> {
    pub fn builder() -> StreamSpecBuilder<C> {
        StreamSpecBuilder::new()
    }

    pub fn with_tag_type<T>(self) -> StreamSpec<T>
    where
        T: CodecTag,
        C: Into<T>,
    {
        StreamSpec {
            codec: self.codec.into(),
            avg_byte_rate: self.avg_byte_rate,
            block_align: self.block_align,
            sample_type: self.sample_type,
            decoded_spec: self.decoded_spec,
        }
    }

    pub fn avg_byte_rate_duration(&self) -> Duration {
        let seconds = 1.0 / self.avg_byte_rate as f64;
        Duration::from_secs_f64(seconds)
    }

    pub fn block_align_duration(&self) -> Duration {
        let seconds = self.block_align as f64 / self.avg_byte_rate as f64;
        Duration::from_secs_f64(seconds)
    }

    pub fn merge<T>(&mut self, other: &StreamSpec<T>) -> PhonicResult<()>
    where
        T: CodecTag + TryInto<C>,
        PhonicError: From<<T as TryInto<C>>::Error>,
    {
        let min_align = self.block_align.min(other.block_align);
        let max_align = self.block_align.max(other.block_align);

        if self.codec != other.codec.try_into()?
            || self.avg_byte_rate != other.avg_byte_rate
            || self.sample_type != other.sample_type
            || max_align % min_align != 0
        {
            todo!()
        }

        self.block_align = max_align;
        self.decoded_spec.merge(&other.decoded_spec)
    }

    pub fn merged<T>(mut self, other: &StreamSpec<T>) -> PhonicResult<Self>
    where
        T: CodecTag + TryInto<C>,
        PhonicError: From<<T as TryInto<C>>::Error>,
    {
        self.merge(other)?;
        Ok(self)
    }
}

impl<C: CodecTag> StreamSpecBuilder<C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tag_type<T>(self) -> PhonicResult<StreamSpecBuilder<T>>
    where
        T: CodecTag,
        C: TryInto<T>,
        PhonicError: From<<C as TryInto<T>>::Error>,
    {
        Ok(StreamSpecBuilder {
            codec: self.codec.map(TryInto::try_into).transpose()?,
            avg_byte_rate: self.avg_byte_rate,
            block_align: self.block_align,
            sample_type: self.sample_type,
            decoded_spec: self.decoded_spec,
        })
    }

    pub fn with_codec(mut self, codec: C) -> Self {
        self.codec = Some(codec);
        self
    }

    pub fn with_avg_byte_rate(mut self, byte_rate: u32) -> Self {
        self.avg_byte_rate = Some(byte_rate);
        self
    }

    pub fn with_block_align(mut self, block_align: usize) -> Self {
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

    pub fn with_decoded_spec(mut self, decoded_spec: impl Into<SignalSpecBuilder>) -> Self {
        self.decoded_spec = decoded_spec.into();
        self
    }

    pub fn is_full(&self) -> bool {
        self.avg_byte_rate.is_some()
            && self.block_align.is_some()
            && self.sample_type.is_some()
            && self.decoded_spec.is_full()
    }

    pub fn is_empty(&self) -> bool {
        self.avg_byte_rate.is_none()
            && self.block_align.is_none()
            && self.sample_type.is_none()
            && self.decoded_spec.is_empty()
    }

    pub fn merge(&mut self, other: &Self) -> PhonicResult<()> {
        if let Some(codec) = other.codec {
            if self.codec.get_or_insert(codec) != &codec {
                // return Err(PhonicError::SignalMismatch);
                todo!()
            }
        }

        if let Some(byte_rate) = other.avg_byte_rate {
            if self.avg_byte_rate.get_or_insert(byte_rate) != &byte_rate {
                // return Err(PhonicError::SignalMismatch);
                todo!()
            }
        }

        if let Some(block_align) = other.block_align {
            if self
                .block_align
                .is_some_and(|align| block_align % align != 0)
            {
                // return Err(PhonicError::SignalMismatch);
                todo!()
            }

            self.block_align = Some(block_align);
        }

        self.decoded_spec.merge(&other.decoded_spec)
    }

    pub fn merged(mut self, other: &Self) -> PhonicResult<Self> {
        self.merge(other)?;
        Ok(self)
    }

    pub fn infer(&mut self) -> PhonicResult<()> {
        C::infer_spec(self)
    }

    pub fn inferred(mut self) -> PhonicResult<Self> {
        self.infer()?;
        Ok(self)
    }

    pub fn build(self) -> PhonicResult<StreamSpec<C>> {
        self.try_into()
    }
}

impl<C: CodecTag> Default for StreamSpecBuilder<C> {
    fn default() -> Self {
        Self {
            codec: Default::default(),
            avg_byte_rate: Default::default(),
            block_align: Default::default(),
            sample_type: Default::default(),
            decoded_spec: Default::default(),
        }
    }
}

impl<C: CodecTag> TryFrom<StreamSpecBuilder<C>> for StreamSpec<C> {
    type Error = PhonicError;

    fn try_from(spec: StreamSpecBuilder<C>) -> Result<StreamSpec<C>, Self::Error> {
        Ok(StreamSpec {
            codec: spec.codec.ok_or(PhonicError::MissingData)?,
            avg_byte_rate: spec.avg_byte_rate.ok_or(PhonicError::MissingData)?,
            block_align: spec.block_align.ok_or(PhonicError::MissingData)?,
            sample_type: spec.sample_type.ok_or(PhonicError::MissingData)?,
            decoded_spec: spec.decoded_spec.build()?,
        })
    }
}

impl<C: CodecTag> From<StreamSpec<C>> for StreamSpecBuilder<C> {
    fn from(spec: StreamSpec<C>) -> Self {
        Self {
            codec: spec.codec.into(),
            avg_byte_rate: spec.avg_byte_rate.into(),
            block_align: spec.block_align.into(),
            sample_type: spec.sample_type.into(),
            decoded_spec: spec.decoded_spec.into(),
        }
    }
}

impl<T: Signal, C: CodecTag> From<&T> for StreamSpecBuilder<C> {
    fn from(value: &T) -> Self {
        Self {
            sample_type: TypeId::of::<T::Sample>().into(),
            decoded_spec: (*value.spec()).into(),
            ..Self::default()
        }
    }
}

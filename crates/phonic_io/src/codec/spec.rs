use crate::{CodecTag, TypeLayout};
use phonic_signal::{PhonicError, PhonicResult, Sample, Signal, SignalSpec, SignalSpecBuilder};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
pub struct StreamSpec<C: CodecTag> {
    pub codec: C,
    pub byte_rate: usize,
    pub block_align: usize,
    pub sample: TypeLayout,
    pub decoded: SignalSpec,
}

#[derive(Debug, Clone, Copy)]
pub struct StreamSpecBuilder<C: CodecTag> {
    pub codec: Option<C>,
    pub byte_rate: Option<usize>,
    pub block_align: Option<usize>,
    pub sample: Option<TypeLayout>,
    pub decoded: SignalSpecBuilder,
}

impl<C: CodecTag> StreamSpec<C> {
    pub fn builder() -> StreamSpecBuilder<C> {
        StreamSpecBuilder::new()
    }

    pub fn into_builder(self) -> StreamSpecBuilder<C> {
        self.into()
    }

    pub fn with_tag_type<T>(self) -> StreamSpec<T>
    where
        T: CodecTag,
        C: Into<T>,
    {
        StreamSpec {
            codec: self.codec.into(),
            byte_rate: self.byte_rate,
            block_align: self.block_align,
            sample: self.sample,
            decoded: self.decoded,
        }
    }

    pub fn try_with_tag_type<T>(self) -> Result<StreamSpec<T>, C::Error>
    where
        T: CodecTag,
        C: TryInto<T>,
    {
        Ok(StreamSpec {
            codec: self.codec.try_into()?,
            byte_rate: self.byte_rate,
            block_align: self.block_align,
            sample: self.sample,
            decoded: self.decoded,
        })
    }

    pub fn merge(&mut self, other: &Self) -> PhonicResult<()> {
        let min_align = self.block_align.min(other.block_align);
        let max_align = self.block_align.max(other.block_align);

        if self.codec != other.codec
            || self.byte_rate != other.byte_rate
            || self.sample != other.sample
            || max_align % min_align != 0
            || self.decoded != other.decoded
        {
            return Err(PhonicError::param_mismatch());
        }

        self.block_align = max_align;

        Ok(())
    }

    pub fn merged(mut self, other: &Self) -> PhonicResult<Self> {
        self.merge(other)?;
        Ok(self)
    }
}

impl<C: CodecTag> StreamSpecBuilder<C> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tag_type<T>(self) -> StreamSpecBuilder<T>
    where
        T: CodecTag,
        C: Into<T>,
    {
        StreamSpecBuilder {
            codec: self.codec.map(Into::into),
            byte_rate: self.byte_rate,
            block_align: self.block_align,
            sample: self.sample,
            decoded: self.decoded,
        }
    }

    pub fn with_optional_tag_type<T>(self) -> StreamSpecBuilder<T>
    where
        T: CodecTag,
        C: Into<Option<T>>,
    {
        StreamSpecBuilder {
            codec: self.codec.and_then(Into::into),
            byte_rate: self.byte_rate,
            block_align: self.block_align,
            sample: self.sample,
            decoded: self.decoded,
        }
    }

    pub fn try_with_tag_type<T>(self) -> Result<StreamSpecBuilder<T>, C::Error>
    where
        T: CodecTag,
        C: TryInto<T>,
    {
        Ok(StreamSpecBuilder {
            codec: self.codec.map(TryInto::try_into).transpose()?,
            byte_rate: self.byte_rate,
            block_align: self.block_align,
            sample: self.sample,
            decoded: self.decoded,
        })
    }

    pub fn with_codec(mut self, codec: impl Into<Option<C>>) -> Self {
        self.codec = codec.into();
        self
    }

    pub fn with_byte_rate(mut self, byte_rate: impl Into<Option<usize>>) -> Self {
        self.byte_rate = byte_rate.into();
        self
    }

    pub fn with_block_align(mut self, block_align: impl Into<Option<usize>>) -> Self {
        self.block_align = block_align.into();
        self
    }

    pub fn with_sample_layout(mut self, layout: impl Into<Option<TypeLayout>>) -> Self {
        self.sample = layout.into();
        self
    }

    pub fn with_sample_type<T: Sample + 'static>(self) -> Self {
        self.with_sample_layout(TypeLayout::of::<T>())
    }

    pub fn with_decoded_spec(mut self, decoded_spec: impl Into<SignalSpecBuilder>) -> Self {
        self.decoded = decoded_spec.into();
        self
    }

    pub fn with_decoded_sample_rate(mut self, sample_rate: impl Into<Option<usize>>) -> Self {
        self.decoded.sample_rate = sample_rate.into();
        self
    }

    pub fn with_decoded_channels(mut self, n_channels: impl Into<Option<usize>>) -> Self {
        self.decoded.n_channels = n_channels.into();
        self
    }

    pub fn is_full(&self) -> bool {
        self.byte_rate.is_some()
            && self.block_align.is_some()
            && self.sample.is_some()
            && self.decoded.is_full()
    }

    pub fn is_empty(&self) -> bool {
        self.byte_rate.is_none()
            && self.block_align.is_none()
            && self.sample.is_none()
            && self.decoded.is_empty()
    }

    pub fn merge(&mut self, other: &Self) -> PhonicResult<()> {
        if other
            .codec
            .is_some_and(|codec| *self.codec.get_or_insert(codec) != codec)
        {
            return Err(PhonicError::param_mismatch());
        }

        if other
            .byte_rate
            .is_some_and(|rate| *self.byte_rate.get_or_insert(rate) != rate)
        {
            return Err(PhonicError::param_mismatch());
        }

        if let Some(align) = other.block_align {
            let self_align = self.block_align.unwrap_or(align);
            let min = align.min(self_align);
            let max = align.max(self_align);

            if max % min != 0 {
                return Err(PhonicError::param_mismatch());
            }

            self.block_align = Some(max);
        }

        self.decoded.merge(&other.decoded)
    }

    pub fn merged(mut self, other: &Self) -> PhonicResult<Self> {
        self.merge(other)?;
        Ok(self)
    }

    pub fn inferred(self) -> PhonicResult<StreamSpec<C>> {
        C::infer_spec(self)
    }

    pub fn build(self) -> PhonicResult<StreamSpec<C>> {
        self.try_into()
    }
}

impl<C: CodecTag> Default for StreamSpecBuilder<C> {
    fn default() -> Self {
        Self {
            codec: Default::default(),
            byte_rate: Default::default(),
            block_align: Default::default(),
            sample: Default::default(),
            decoded: Default::default(),
        }
    }
}

impl<C: CodecTag> TryFrom<StreamSpecBuilder<C>> for StreamSpec<C> {
    type Error = PhonicError;

    fn try_from(spec: StreamSpecBuilder<C>) -> Result<StreamSpec<C>, Self::Error> {
        Ok(StreamSpec {
            codec: spec.codec.ok_or(PhonicError::missing_data())?,
            byte_rate: spec.byte_rate.ok_or(PhonicError::missing_data())?,
            block_align: spec.block_align.ok_or(PhonicError::missing_data())?,
            sample: spec.sample.ok_or(PhonicError::missing_data())?,
            decoded: spec.decoded.build()?,
        })
    }
}

impl<C: CodecTag> From<StreamSpec<C>> for StreamSpecBuilder<C> {
    fn from(spec: StreamSpec<C>) -> Self {
        Self {
            codec: spec.codec.into(),
            byte_rate: spec.byte_rate.into(),
            block_align: spec.block_align.into(),
            sample: spec.sample.into(),
            decoded: spec.decoded.into(),
        }
    }
}

impl<T: Signal, C: CodecTag> From<&T> for StreamSpecBuilder<C> {
    fn from(value: &T) -> Self {
        Self {
            sample: Some(TypeLayout::of::<T::Sample>()),
            decoded: value.spec().into_builder(),
            ..Self::default()
        }
    }
}

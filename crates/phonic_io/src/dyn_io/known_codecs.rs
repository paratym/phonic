use crate::{
    dyn_io::{DynCodecConstructor, DynStream, TaggedSignal},
    CodecTag, StreamSpec, StreamSpecBuilder,
};
use phonic_signal::{PhonicError, PhonicResult};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum KnownCodec {
    #[cfg(feature = "pcm")]
    Pcm,
}

impl CodecTag for KnownCodec {
    fn infer_spec(spec: StreamSpecBuilder<Self>) -> PhonicResult<StreamSpec<Self>> {
        use crate::codec::*;

        match spec.codec {
            #[cfg(feature = "pcm")]
            Some(Self::Pcm) => pcm::PcmCodecTag::infer_tagged_spec(spec),

            None => Err(PhonicError::MissingData),
        }
    }
}

impl DynCodecConstructor for KnownCodec {
    fn encoder(&self, signal: TaggedSignal) -> PhonicResult<Box<dyn DynStream<Tag = Self>>> {
        use crate::codec::*;

        match self {
            #[cfg(feature = "pcm")]
            Self::Pcm => pcm::PcmCodecTag::dyn_encoder(signal),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }

    fn decoder<S: DynStream<Tag = Self> + 'static>(stream: S) -> PhonicResult<TaggedSignal> {
        use crate::codec::*;

        match stream.stream_spec().codec {
            #[cfg(feature = "pcm")]
            Self::Pcm => pcm::PcmCodecTag::dyn_decoder(Box::new(stream)),

            #[allow(unreachable_patterns)]
            _ => Err(PhonicError::Unsupported),
        }
    }
}

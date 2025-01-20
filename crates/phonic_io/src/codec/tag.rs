use crate::{Stream, StreamSpec, StreamSpecBuilder};
use phonic_signal::{PhonicResult, Signal};
use std::{fmt::Debug, hash::Hash};

pub trait CodecTag: Sized + Send + Sync + Debug + Copy + Eq + Hash {
    fn infer_spec(spec: StreamSpecBuilder<Self>) -> PhonicResult<StreamSpec<Self>>;
}

pub trait CodecFromSignal<T: Signal, C: CodecTag>: Sized + Stream<Tag = C> {
    fn from_signal(tag: C, inner: T) -> PhonicResult<Self>;

    fn default_from_signal(inner: T) -> PhonicResult<Self>
    where
        C: Default,
    {
        Self::from_signal(C::default(), inner)
    }
}

pub trait CodecFromStream<T: Stream<Tag = C>, C: CodecTag>: Sized + Signal {
    fn from_stream(inner: T) -> PhonicResult<Self>;
}

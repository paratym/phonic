use crate::{Stream, StreamSpec, StreamSpecBuilder};
use phonic_signal::{PhonicResult, Signal};
use std::{fmt::Debug, hash::Hash};

pub trait CodecTag: Sized + Send + Sync + Debug + Copy + Eq + Hash {
    fn infer_spec(spec: StreamSpecBuilder<Self>) -> PhonicResult<StreamSpec<Self>>;
}

pub trait Encoder<T, C: CodecTag>: Sized + Stream<Tag = C> {
    fn encoder(inner: T) -> PhonicResult<Self>
    where
        T: Signal;
}

pub trait Decoder<T, C: CodecTag>: Sized + Signal {
    fn decoder(inner: T) -> PhonicResult<Self>
    where
        T: Stream<Tag = C>;
}

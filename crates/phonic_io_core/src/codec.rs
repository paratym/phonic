use crate::{Stream, StreamSpecBuilder};
use phonic_signal::{PhonicError, PhonicResult, Signal};
use std::{fmt::Debug, hash::Hash};

pub trait CodecTag: Sized + Send + Sync + Debug + Copy + Eq + Hash {
    fn infer_spec(spec: &mut StreamSpecBuilder<Self>) -> PhonicResult<()>;
}

pub trait CodecConstructor<T, C: CodecTag>: Sized {
    fn encoder(inner: T) -> PhonicResult<Self>
    where
        Self: Stream<Tag = C>,
        T: Signal;

    fn decoder(inner: T) -> PhonicResult<Self>
    where
        Self: Signal,
        T: Stream,
        T::Tag: TryInto<C>,
        PhonicError: From<<T::Tag as TryInto<C>>::Error>;
}

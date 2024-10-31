use crate::{Stream, StreamSpecBuilder};
use phonic_core::PhonicError;
use phonic_signal::Signal;
use std::{fmt::Debug, hash::Hash};

pub trait CodecTag: Sized + Send + Sync + Debug + Copy + Eq + Hash {
    fn infer_spec(spec: &mut StreamSpecBuilder<Self>) -> Result<(), PhonicError>;
}

pub trait CodecConstructor<T, C: CodecTag>: Sized {
    fn encoder(inner: T) -> Result<Self, PhonicError>
    where
        Self: Stream<Tag = C>,
        T: Signal;

    fn decoder(inner: T) -> Result<Self, PhonicError>
    where
        Self: Signal,
        T: Stream,
        T::Tag: TryInto<C>,
        PhonicError: From<<T::Tag as TryInto<C>>::Error>;
}

use crate::{CodecTag, Format, StreamSpec};
use phonic_signal::PhonicResult;
use std::{
    fmt::Debug,
    hash::Hash,
    io::{Read, Write},
};

pub trait FormatTag: Sized + Send + Sync + Debug + Copy + Eq + Hash {
    type Codec: CodecTag;
}

pub trait FormatConstructor<T, F: FormatTag>: Sized {
    fn read_index(inner: T) -> PhonicResult<Self>
    where
        Self: Format<Tag = F>,
        T: Read;

    fn write_index<I>(inner: T, index: I) -> PhonicResult<Self>
    where
        Self: Format<Tag = F>,
        T: Write,
        I: IntoIterator<Item = StreamSpec<F::Codec>>;
}

use crate::{CodecTag, Format, StreamSpec};
use phonic_signal::PhonicResult;
use std::{fmt::Debug, hash::Hash};

pub trait FormatTag: Sized + Send + Sync + Debug + Copy + Eq + Hash {
    type Codec: CodecTag;
}

pub trait FormatFromReader<R, F: FormatTag>: Sized + Format<Tag = F> {
    fn read_index(reader: R) -> PhonicResult<Self>;
}

pub trait FormatFromWriter<W, F: FormatTag>: Sized + Format<Tag = F> {
    fn write_index<I>(writer: W, index: I) -> PhonicResult<Self>
    where
        I: IntoIterator<Item = StreamSpec<F::Codec>>;
}

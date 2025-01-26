use crate::{
    dyn_io::{DynFormat, DynFormatConstructor, KnownCodec, StdIoSource},
    utils::PollIo,
    FormatFromReader, FormatFromWriter, FormatTag, StreamSpec,
};
use phonic_signal::PhonicResult;

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
#[non_exhaustive]
pub enum KnownFormat {
    #[cfg(feature = "wave")]
    Wave,
}

impl FormatTag for KnownFormat {
    type Codec = KnownCodec;
}

impl DynFormatConstructor for KnownFormat {
    fn read_index<T>(&self, inner: T) -> PhonicResult<Box<dyn DynFormat<Tag = Self>>>
    where
        T: StdIoSource + 'static,
    {
        use crate::formats::*;

        Ok(match self {
            #[cfg(feature = "wave")]
            Self::Wave => Box::new(PollIo(wave::WaveFormat::read_index(inner)?)),
        })
    }

    fn write_index<T, I>(&self, inner: T, index: I) -> PhonicResult<Box<dyn DynFormat<Tag = Self>>>
    where
        T: StdIoSource + 'static,
        I: IntoIterator<Item = StreamSpec<Self::Codec>>,
    {
        use crate::formats::*;

        Ok(match self {
            #[cfg(feature = "wave")]
            Self::Wave => Box::new(PollIo(wave::WaveFormat::write_index(inner, index)?)),
        })
    }
}

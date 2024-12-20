use crate::{utils::StreamSelector, Format};

pub trait UtilFormatExt: Sized + Format {
    fn into_stream(self, stream: usize) -> StreamSelector<Self>
    where
        Self: Sized,
    {
        StreamSelector::new(self, stream)
    }

    fn into_current_stream(self) -> StreamSelector<Self>
    where
        Self: Sized,
    {
        let i = self.current_stream();
        self.into_stream(i)
    }

    fn into_primary_stream(self) -> Option<StreamSelector<Self>>
    where
        Self: Sized,
    {
        self.primary_stream().map(|i| self.into_stream(i))
    }
}

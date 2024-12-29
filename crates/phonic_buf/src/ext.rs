use crate::{DynamicBuf, ResizeBuf, SizedBuf, SpecBuf};
use phonic_signal::{BlockingSignalReader, PhonicResult, Signal, SignalDuration};

pub trait BufExt: Sized + Signal {
    fn read_into<T>(&mut self) -> PhonicResult<T>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        T::read(self)
    }

    fn read_into_sized<T>(&mut self) -> PhonicResult<T>
    where
        Self: BlockingSignalReader,
        T: SizedBuf<Item = Self::Sample>,
    {
        T::read(self)
    }

    fn read_into_exact<T, D>(&mut self, duration: D) -> PhonicResult<T>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        D: SignalDuration,
    {
        T::read_exact(self, duration)
    }

    fn read_all_into<T>(&mut self) -> PhonicResult<T>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        T::read_all(self)
    }

    fn take<T>(&mut self) -> PhonicResult<SpecBuf<T, Self::Sample>>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        SpecBuf::read(self)
    }

    fn take_sized<T>(&mut self) -> PhonicResult<SpecBuf<T, Self::Sample>>
    where
        Self: BlockingSignalReader,
        T: SizedBuf<Item = Self::Sample>,
    {
        SpecBuf::read_sized(self)
    }

    fn take_exact<T, D>(&mut self, duration: D) -> PhonicResult<SpecBuf<T, Self::Sample>>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        D: SignalDuration,
    {
        SpecBuf::read_exact(self, duration)
    }

    fn take_all<T>(&mut self) -> PhonicResult<SpecBuf<T, Self::Sample>>
    where
        Self: BlockingSignalReader,
        T: DynamicBuf<Item = Self::Sample>,
        T::Uninit: ResizeBuf,
    {
        SpecBuf::read_all(self)
    }
}

use crate::{
    delegate_signal, BlockingSignalReader, BlockingSignalWriter, NFrames, PhonicResult, Signal,
    SignalDuration, SignalReader, SignalSeeker, SignalWriter,
};
use std::mem::MaybeUninit;

pub struct Observer<T: Signal> {
    inner: T,
    callback: Callback<T>,
}

#[allow(clippy::type_complexity)]
enum Callback<T: Signal> {
    Event(Box<dyn for<'e> Fn(&T, SignalEvent<'e, T>)>),
    Read(Box<dyn Fn(&T, &[T::Sample])>),
    Write(Box<dyn Fn(&T, &[T::Sample])>),
    Seek(Box<dyn Fn(&T, i64)>),
}

pub enum SignalEvent<'a, T: Signal> {
    Read(&'a [T::Sample]),
    Write(&'a [T::Sample]),
    Seek(i64),
}

impl<T: Signal> Observer<T> {
    pub fn new<F>(inner: T, callback: F) -> Self
    where
        F: for<'e> Fn(&T, SignalEvent<'e, T>) + 'static,
    {
        Self {
            inner,
            callback: Callback::Event(Box::new(callback)),
        }
    }

    pub fn on_read<F>(inner: T, callback: F) -> Self
    where
        F: Fn(&T, &[T::Sample]) + 'static,
    {
        Self {
            inner,
            callback: Callback::Read(Box::new(callback)),
        }
    }

    pub fn on_write<F>(inner: T, callback: F) -> Self
    where
        F: Fn(&T, &[T::Sample]) + 'static,
    {
        Self {
            inner,
            callback: Callback::Write(Box::new(callback)),
        }
    }

    pub fn on_seek<F>(inner: T, callback: F) -> Self
    where
        F: Fn(&T, i64) + 'static,
    {
        Self {
            inner,
            callback: Callback::Seek(Box::new(callback)),
        }
    }
}

impl<T: Signal> Callback<T> {
    pub fn on_read(&self, signal: &T, samples: &[T::Sample]) {
        match self {
            Self::Read(callback) => callback(signal, samples),
            Self::Event(callback) => callback(signal, SignalEvent::Read(samples)),
            _ => {}
        }
    }

    pub fn on_write(&self, signal: &T, samples: &[T::Sample]) {
        match self {
            Self::Write(callback) => callback(signal, samples),
            Self::Event(callback) => callback(signal, SignalEvent::Write(samples)),
            _ => {}
        }
    }

    pub fn on_seek(&self, signal: &T, offset: i64) {
        match self {
            Self::Seek(callback) => callback(signal, offset),
            Self::Event(callback) => callback(signal, SignalEvent::Seek(offset)),
            _ => {}
        }
    }
}

delegate_signal! {
    delegate<T> * + !Mut for Observer<T> {
        Self as T;

        &self => &self.inner;
    }
}

impl<T: SignalReader> SignalReader for Observer<T> {
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let samples = self.inner.read_init(buf)?;
        self.callback.on_read(&self.inner, samples);

        Ok(samples.len())
    }
}

impl<T: BlockingSignalReader> BlockingSignalReader for Observer<T> {
    fn read_blocking(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let samples = self.inner.read_init_blocking(buf)?;
        self.callback.on_read(&self.inner, samples);

        Ok(samples.len())
    }

    fn read_exact(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<()> {
        let samples = self.inner.read_exact_init(buf)?;
        self.callback.on_read(&self.inner, samples);

        Ok(())
    }
}

impl<T: SignalWriter> SignalWriter for Observer<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let n = self.inner.write(buf)?;
        self.callback.on_write(&self.inner, &buf[..n]);

        Ok(n)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        self.inner.flush()
    }
}

impl<T: BlockingSignalWriter> BlockingSignalWriter for Observer<T> {
    fn write_blocking(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let n = self.inner.write_blocking(buf)?;
        self.callback.on_write(&self.inner, &buf[..n]);

        Ok(n)
    }

    fn write_exact(&mut self, buf: &[Self::Sample]) -> PhonicResult<()> {
        self.inner.write_exact(buf)?;
        self.callback.on_write(&self.inner, buf);

        Ok(())
    }

    fn flush_blocking(&mut self) -> PhonicResult<()> {
        self.inner.flush_blocking()
    }
}

impl<T: SignalSeeker> SignalSeeker for Observer<T> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        self.inner.seek(offset)?;
        self.callback.on_seek(&self.inner, offset);

        Ok(())
    }
}

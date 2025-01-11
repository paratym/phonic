use crate::{
    delegate_signal, BufferedSignalReader, BufferedSignalWriter, PhonicResult, Signal, SignalExt,
    SignalReader, SignalSeeker, SignalWriter,
};
use std::mem::MaybeUninit;

pub struct Observer<T: Signal> {
    inner: T,
    callback: Callback<T>,
}

#[allow(clippy::type_complexity)]
enum Callback<T: Signal> {
    Event(Box<dyn for<'a> Fn(&T, SignalEvent<'a, T>)>),
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
    impl<T> * + !Mut for Observer<T> {
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

impl<T: BufferedSignalReader> BufferedSignalReader for Observer<T> {
    fn fill(&mut self) -> PhonicResult<&[Self::Sample]> {
        self.inner.fill()
    }

    fn buffer(&self) -> Option<&[Self::Sample]> {
        self.inner.buffer()
    }

    fn consume(&mut self, n_samples: usize) {
        let samples = &self.inner.buffer().unwrap()[..n_samples];
        self.callback.on_read(&self.inner, samples);

        self.inner.consume(n_samples)
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

impl<T: BufferedSignalWriter> BufferedSignalWriter for Observer<T> {
    fn buffer_mut(&mut self) -> Option<&mut [MaybeUninit<Self::Sample>]> {
        self.inner.buffer_mut()
    }

    fn commit(&mut self, n_samples: usize) {
        // let uninit_samples = &self.inner.buffer_mut().unwrap()[..n_samples];
        // let samples = unsafe { slice_as_init(uninit_samples) };
        // self.callback.on_write(&self.inner, samples);

        self.inner.commit(n_samples)
    }
}

impl<T: SignalSeeker> SignalSeeker for Observer<T> {
    fn seek(&mut self, offset: i64) -> PhonicResult<()> {
        self.inner.seek(offset)?;
        self.callback.on_seek(&self.inner, offset);

        Ok(())
    }
}

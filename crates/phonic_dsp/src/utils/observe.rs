use phonic_core::PhonicError;
use phonic_signal::{
    FiniteSignal, IndexedSignal, Signal, SignalReader, SignalSeeker, SignalSpec, SignalWriter,
};

pub enum SignalEvent<'b, T: Signal> {
    Read(&'b [T::Sample]),
    Write(&'b [T::Sample]),
    Seek(i64),
}

#[allow(clippy::type_complexity)]
enum Callback<T: Signal> {
    Event(Box<dyn for<'s, 'b> Fn(&'s T, SignalEvent<'b, T>)>),
    Read(Box<dyn for<'s, 'b> Fn(&'s T, &'b [T::Sample])>),
    Write(Box<dyn for<'s, 'b> Fn(&T, &[T::Sample])>),
    Seek(Box<dyn Fn(&T, i64)>),
}

pub struct Observer<T: Signal> {
    inner: T,
    callback: Callback<T>,
}

impl<T: Signal> Observer<T> {
    pub fn new<F>(inner: T, callback: F) -> Self
    where
        F: for<'s, 'b> Fn(&'s T, SignalEvent<'b, T>) + 'static,
    {
        Self {
            inner,
            callback: Callback::Event(Box::new(callback)),
        }
    }

    pub fn on_read<F>(inner: T, callback: F) -> Self
    where
        F: for<'s, 'b> Fn(&'s T, &'b [T::Sample]) + 'static,
    {
        Self {
            inner,
            callback: Callback::Read(Box::new(callback)),
        }
    }

    pub fn on_write<F>(inner: T, callback: F) -> Self
    where
        F: for<'s, 'b> Fn(&'s T, &'b [T::Sample]) + 'static,
    {
        Self {
            inner,
            callback: Callback::Write(Box::new(callback)),
        }
    }

    pub fn on_seek<F>(inner: T, callback: F) -> Self
    where
        F: for<'s> Fn(&'s T, i64) + 'static,
    {
        Self {
            inner,
            callback: Callback::Seek(Box::new(callback)),
        }
    }
}

impl<T: Signal> Signal for Observer<T> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        self.inner.spec()
    }
}

impl<T: IndexedSignal> IndexedSignal for Observer<T> {
    fn pos(&self) -> u64 {
        self.inner.pos()
    }
}

impl<T: FiniteSignal> FiniteSignal for Observer<T> {
    fn len(&self) -> u64 {
        self.inner.len()
    }
}

impl<T: SignalReader> SignalReader for Observer<T> {
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let n = self.inner.read(buf)?;

        match &self.callback {
            Callback::Read(callback) => callback(&self.inner, &buf[..n]),
            Callback::Event(callback) => callback(&self.inner, SignalEvent::Read(&buf[..n])),
            _ => {}
        }

        Ok(n)
    }
}

impl<T: SignalWriter> SignalWriter for Observer<T> {
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, PhonicError> {
        let n = self.inner.write(buf)?;

        match &self.callback {
            Callback::Write(callback) => callback(&self.inner, &buf[..n]),
            Callback::Event(callback) => callback(&self.inner, SignalEvent::Write(&buf[..n])),
            _ => {}
        }

        Ok(n)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        self.inner.flush()
    }
}

impl<T: SignalSeeker> SignalSeeker for Observer<T> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        self.inner.seek(offset)?;

        match &self.callback {
            Callback::Seek(callback) => callback(&self.inner, offset),
            Callback::Event(callback) => callback(&self.inner, SignalEvent::Seek(offset)),
            _ => {}
        }

        Ok(())
    }
}
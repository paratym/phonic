use crate::{
    utils::{IntoDuration, NSamples},
    BlockingSignal, FiniteSignal, PhonicError, PhonicResult, Sample, SignalExt, SignalReader,
};
use std::{
    mem::{transmute, MaybeUninit},
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};

pub const DEFAULT_BUF_LEN: usize = 4096;
pub type DefaultSizedBuf<T, const N: usize = DEFAULT_BUF_LEN> = [T; N];
pub type DefaultDynamicBuf<T> = Box<[T]>;

pub trait OwnedBuf: Sized {
    type Item;

    type Uninit: OwnedBuf<Item = MaybeUninit<Self::Item>>;
    unsafe fn from_uninit(uninit: Self::Uninit) -> Self;

    fn _as_slice(&self) -> &[Self::Item];
    fn _as_mut_slice(&mut self) -> &mut [Self::Item];
}

pub trait SizedBuf: OwnedBuf {
    fn uninit() -> Self::Uninit;

    fn silence() -> Self
    where
        Self::Item: Sample,
    {
        let mut buf = Self::uninit();
        buf._as_mut_slice()
            .fill(MaybeUninit::new(Self::Item::ORIGIN));

        unsafe { Self::from_uninit(buf) }
    }

    fn read<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: BlockingSignal + SignalReader<Sample = Self::Item>,
    {
        let mut buf = Self::uninit();
        reader.read_exact(buf._as_mut_slice())?;

        Ok(unsafe { Self::from_uninit(buf) })
    }
}

pub trait DynamicBuf: OwnedBuf {
    fn uninit(len: usize) -> Self::Uninit;

    fn silence(len: usize) -> Self
    where
        Self::Item: Sample,
    {
        let mut buf = Self::uninit(len);
        buf._as_mut_slice()
            .fill(MaybeUninit::new(Self::Item::ORIGIN));

        unsafe { Self::from_uninit(buf) }
    }

    fn read<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: SignalReader<Sample = Self::Item>,
        Self::Uninit: ResizeBuf,
    {
        let mut buf = Self::uninit(DEFAULT_BUF_LEN);
        let n_samples = reader.read(buf._as_mut_slice())?;
        unsafe { buf.resize(n_samples) }

        Ok(unsafe { Self::from_uninit(buf) })
    }

    fn read_exact<R>(reader: &mut R, duration: impl IntoDuration<NSamples>) -> PhonicResult<Self>
    where
        R: BlockingSignal + SignalReader<Sample = Self::Item>,
    {
        let NSamples { n_samples } = duration.into_duration(reader.spec());
        debug_assert_eq!(n_samples % reader.spec().channels.count() as u64, 0);

        let mut buf = Self::uninit(n_samples as usize);
        reader.read_exact(buf._as_mut_slice())?;

        Ok(unsafe { Self::from_uninit(buf) })
    }

    fn read_all<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: BlockingSignal + SignalReader<Sample = Self::Item>,
        Self::Uninit: ResizeBuf,
    {
        let mut buf = Self::uninit(DEFAULT_BUF_LEN);
        let mut i = 0;

        loop {
            let slice = buf._as_mut_slice();
            match reader.read(&mut slice[i..]) {
                Ok(0) => break,
                Ok(n) => i += n,
                Err(PhonicError::Interrupted) => continue,
                Err(PhonicError::NotReady) => {
                    reader.block();
                    continue;
                }
                Err(e) => return Err(e),
            }

            if slice.len() - i < DEFAULT_BUF_LEN {
                let len = slice.len() + DEFAULT_BUF_LEN;
                unsafe { buf.resize(len) };
            }
        }

        let init_buf = unsafe {
            buf.resize(i);
            Self::from_uninit(buf)
        };

        Ok(init_buf)
    }
}

pub trait ResizeBuf: OwnedBuf {
    unsafe fn resize(&mut self, len: usize);
}

impl<T, const N: usize> OwnedBuf for [T; N] {
    type Item = T;
    type Uninit = [MaybeUninit<T>; N];

    unsafe fn from_uninit(uninit: Self::Uninit) -> Self {
        let init_ptr = uninit.as_ptr().cast();
        unsafe { std::ptr::read(init_ptr) }
    }

    fn _as_slice(&self) -> &[T] {
        self.as_ref()
    }

    fn _as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut()
    }
}

impl<T> OwnedBuf for Vec<T> {
    type Item = T;
    type Uninit = Vec<MaybeUninit<T>>;

    unsafe fn from_uninit(uninit: Self::Uninit) -> Self {
        transmute(uninit)
    }

    fn _as_slice(&self) -> &[T] {
        self.deref()
    }

    fn _as_mut_slice(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

impl<T> OwnedBuf for Box<[T]> {
    type Item = T;
    type Uninit = Vec<MaybeUninit<T>>;

    unsafe fn from_uninit(uninit: Self::Uninit) -> Self {
        transmute(uninit.into_boxed_slice())
    }

    fn _as_slice(&self) -> &[T] {
        self.deref()
    }

    fn _as_mut_slice(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

impl<T> OwnedBuf for Rc<[T]> {
    type Item = T;
    type Uninit = Vec<MaybeUninit<T>>;

    unsafe fn from_uninit(uninit: Self::Uninit) -> Self {
        let rc_uninit: Rc<[MaybeUninit<T>]> = uninit.into();
        transmute(rc_uninit)
    }

    fn _as_slice(&self) -> &[T] {
        self.deref()
    }

    fn _as_mut_slice(&mut self) -> &mut [T] {
        Rc::get_mut(self).unwrap_or(&mut [])
    }
}

impl<T> OwnedBuf for Arc<[T]> {
    type Item = T;
    type Uninit = Vec<MaybeUninit<T>>;

    unsafe fn from_uninit(uninit: Self::Uninit) -> Self {
        let arc_uninit: Arc<[MaybeUninit<T>]> = uninit.into();
        transmute(arc_uninit)
    }

    fn _as_slice(&self) -> &[T] {
        self.deref()
    }

    fn _as_mut_slice(&mut self) -> &mut [T] {
        Arc::get_mut(self).unwrap_or(&mut [])
    }
}

impl<S: Sized, const N: usize> SizedBuf for [S; N] {
    fn uninit() -> Self::Uninit {
        [const { MaybeUninit::uninit() }; N]
    }
}

impl<S> DynamicBuf for Vec<S> {
    fn uninit(capacity: usize) -> Self::Uninit {
        let mut buf = Vec::with_capacity(capacity);
        unsafe { buf.set_len(capacity) }

        buf
    }
}

impl<S> DynamicBuf for Box<[S]> {
    fn uninit(capacity: usize) -> Self::Uninit {
        Vec::uninit(capacity)
    }
}

impl<S> DynamicBuf for Rc<[S]> {
    fn uninit(len: usize) -> Self::Uninit {
        Vec::uninit(len)
    }
}

impl<S> DynamicBuf for Arc<[S]> {
    fn uninit(len: usize) -> Self::Uninit {
        Vec::uninit(len)
    }
}

impl<S> ResizeBuf for Vec<MaybeUninit<S>> {
    unsafe fn resize(&mut self, len: usize) {
        Vec::resize_with(self, len, MaybeUninit::uninit)
    }
}

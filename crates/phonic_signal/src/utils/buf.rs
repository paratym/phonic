use crate::{
    utils::{IntoDuration, NSamples},
    BlockingSignal, PhonicError, PhonicResult, Sample, SignalExt, SignalReader,
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

    /// Creates an instance of `Self` form an instance of `Self::Uninit`.
    ///
    /// # Safety
    /// The caller must ensure all elements of the buffer have been initialized. Calling this
    /// method when any elements are not yet fully initialized causes immediate undefined behavior.
    /// For more details see `MaybeUninit::assume_init`.
    unsafe fn from_uninit(uninit: Self::Uninit) -> Self;

    /// Returns an immutable reference to the elements of the buffer. This method is **NOT** intended
    /// for use in application code.
    ///
    /// # Safety
    /// The caller must ensure that there are no other references to the buffer at the time of
    /// calling.
    unsafe fn _as_slice(&self) -> &[Self::Item];

    /// Returns an mutable reference to the elements of the buffer. This method is **NOT** intended
    /// for use in application code.
    ///
    /// # Safety
    /// The caller must ensure that there are no other references to the buffer at the time of
    /// calling.
    unsafe fn _as_mut_slice(&mut self) -> &mut [Self::Item];
}

pub trait SizedBuf: OwnedBuf {
    fn uninit() -> Self::Uninit;

    fn init(item: Self::Item) -> Self
    where
        Self::Item: Copy,
    {
        let mut buf = Self::uninit();
        let slice = unsafe { buf._as_mut_slice() };
        slice.fill(MaybeUninit::new(item));

        unsafe { Self::from_uninit(buf) }
    }

    fn default() -> Self
    where
        Self::Item: Copy + Default,
    {
        Self::init(Self::Item::default())
    }

    fn silence() -> Self
    where
        Self::Item: Sample,
    {
        Self::init(Self::Item::ORIGIN)
    }

    fn read<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: BlockingSignal + SignalReader<Sample = Self::Item>,
    {
        let mut buf = Self::uninit();
        let slice = unsafe { buf._as_mut_slice() };
        reader.read_exact(slice)?;

        Ok(unsafe { Self::from_uninit(buf) })
    }
}

pub trait ResizeBuf: OwnedBuf {
    unsafe fn _resize(&mut self, len: usize);
}

pub trait DynamicBuf: OwnedBuf {
    fn uninit(len: usize) -> Self::Uninit;

    fn silence(len: usize) -> Self
    where
        Self::Item: Sample,
    {
        let mut buf = Self::uninit(len);
        let slice = unsafe { buf._as_mut_slice() };
        slice.fill(MaybeUninit::new(Self::Item::ORIGIN));

        unsafe { Self::from_uninit(buf) }
    }

    fn read<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: SignalReader<Sample = Self::Item>,
        Self::Uninit: ResizeBuf,
    {
        let mut buf = Self::uninit(DEFAULT_BUF_LEN);
        let slice = unsafe { buf._as_mut_slice() };
        let n_samples = reader.read(slice)?;
        unsafe { buf._resize(n_samples) }

        Ok(unsafe { Self::from_uninit(buf) })
    }

    fn read_exact<R>(reader: &mut R, duration: impl IntoDuration<NSamples>) -> PhonicResult<Self>
    where
        R: BlockingSignal + SignalReader<Sample = Self::Item>,
    {
        let NSamples { n_samples } = duration.into_duration(reader.spec());
        debug_assert_eq!(n_samples % reader.spec().n_channels as u64, 0);

        let mut buf = Self::uninit(n_samples as usize);
        let slice = unsafe { buf._as_mut_slice() };
        reader.read_exact(slice)?;

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
            let slice = unsafe { buf._as_mut_slice() };
            match reader.read(&mut slice[i..]) {
                Ok(0) => break,
                Ok(n) => i += n,
                Err(PhonicError::Interrupted { .. }) => continue,
                Err(PhonicError::NotReady { .. }) => {
                    reader.block();
                    continue;
                }
                Err(e) => return Err(e),
            }

            if slice.len() - i < DEFAULT_BUF_LEN {
                let len = slice.len() + DEFAULT_BUF_LEN;
                unsafe { buf._resize(len) };
            }
        }

        let init_buf = unsafe {
            buf._resize(i);
            Self::from_uninit(buf)
        };

        Ok(init_buf)
    }
}

impl<T, const N: usize> OwnedBuf for [T; N] {
    type Item = T;
    type Uninit = [MaybeUninit<T>; N];

    unsafe fn from_uninit(uninit: Self::Uninit) -> Self {
        let init_ptr = uninit.as_ptr().cast();
        unsafe { std::ptr::read(init_ptr) }
    }

    unsafe fn _as_slice(&self) -> &[T] {
        self.as_ref()
    }

    unsafe fn _as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut()
    }
}

impl<T> OwnedBuf for Vec<T> {
    type Item = T;
    type Uninit = Vec<MaybeUninit<T>>;

    unsafe fn from_uninit(uninit: Self::Uninit) -> Self {
        transmute(uninit)
    }

    unsafe fn _as_slice(&self) -> &[T] {
        self.deref()
    }

    unsafe fn _as_mut_slice(&mut self) -> &mut [T] {
        self.deref_mut()
    }
}

impl<T> OwnedBuf for Box<[T]> {
    type Item = T;
    type Uninit = Vec<MaybeUninit<T>>;

    unsafe fn from_uninit(uninit: Self::Uninit) -> Self {
        transmute(uninit.into_boxed_slice())
    }

    unsafe fn _as_slice(&self) -> &[T] {
        self.deref()
    }

    unsafe fn _as_mut_slice(&mut self) -> &mut [T] {
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

    unsafe fn _as_slice(&self) -> &[T] {
        self.deref()
    }

    unsafe fn _as_mut_slice(&mut self) -> &mut [T] {
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

    unsafe fn _as_slice(&self) -> &[T] {
        self.deref()
    }

    unsafe fn _as_mut_slice(&mut self) -> &mut [T] {
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
    unsafe fn _resize(&mut self, len: usize) {
        Vec::resize_with(self, len, MaybeUninit::uninit)
    }
}

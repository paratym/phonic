use crate::{BlockingSignalReader, NSamples, PhonicResult, Sample, Signal, SignalDuration};
use std::{
    mem::{transmute, MaybeUninit},
    ops::{Deref, DerefMut},
    rc::Rc,
    sync::Arc,
};

pub const DEFAULT_BUF_LEN: usize = 4096;

pub struct DefaultBuf<S, const N: usize = DEFAULT_BUF_LEN> {
    pub buf: [S; N],
}

pub trait OwnedBuf: Sized {
    type Item;

    type Uninit: OwnedBuf<Item = MaybeUninit<Self::Item>>;
    unsafe fn from_uninit(uninit: Self::Uninit) -> Self;

    fn _as_slice(&self) -> &[Self::Item];
    fn _as_mut_slice(&mut self) -> &mut [Self::Item];
}

pub trait SizedBuf: OwnedBuf {
    fn new_uninit() -> Self::Uninit;

    fn silence() -> Self
    where
        Self::Item: Sample,
    {
        let mut buf = Self::new_uninit();
        buf._as_mut_slice()
            .fill(MaybeUninit::new(Self::Item::ORIGIN));

        unsafe { Self::from_uninit(buf) }
    }

    fn read<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: BlockingSignalReader<Sample = Self::Item>,
    {
        let mut buf = Self::new_uninit();
        reader.read_exact(buf._as_mut_slice())?;

        Ok(unsafe { Self::from_uninit(buf) })
    }
}

pub trait DynamicBuf: OwnedBuf {
    fn new_uninit(len: usize) -> Self::Uninit;

    fn silence(len: usize) -> Self
    where
        Self::Item: Sample,
    {
        let mut buf = Self::new_uninit(len);
        buf._as_mut_slice()
            .fill(MaybeUninit::new(Self::Item::ORIGIN));

        unsafe { Self::from_uninit(buf) }
    }

    fn read<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: BlockingSignalReader<Sample = Self::Item>,
        Self::Uninit: ResizeBuf,
    {
        let mut buf = Self::new_uninit(DEFAULT_BUF_LEN);
        let n_samples = reader.read_blocking(buf._as_mut_slice())?;
        unsafe { buf._resize(n_samples) }

        Ok(unsafe { Self::from_uninit(buf) })
    }

    fn read_exact<R, D>(reader: &mut R, duration: D) -> PhonicResult<Self>
    where
        R: BlockingSignalReader<Sample = Self::Item>,
        D: SignalDuration,
    {
        let NSamples { n_samples } = duration.into_duration(reader.spec());
        debug_assert_eq!(n_samples % reader.spec().channels.count() as u64, 0);

        let mut buf = Self::new_uninit(n_samples as usize);
        reader.read_exact(buf._as_mut_slice())?;

        Ok(unsafe { Self::from_uninit(buf) })
    }

    fn read_all<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: BlockingSignalReader<Sample = Self::Item>,
        Self::Uninit: ResizeBuf,
    {
        todo!()
    }
}

pub trait ResizeBuf {
    unsafe fn _resize(&mut self, len: usize);
}

pub fn slice_as_uninit<T>(init: &[T]) -> &[MaybeUninit<T>] {
    unsafe { transmute::<&[T], &[MaybeUninit<T>]>(init) }
}

pub fn slice_as_uninit_mut<T>(init: &mut [T]) -> &mut [MaybeUninit<T>] {
    unsafe { transmute::<&mut [T], &mut [MaybeUninit<T>]>(init) }
}

pub unsafe fn slice_as_init<T>(uninit: &[MaybeUninit<T>]) -> &[T] {
    transmute::<&[MaybeUninit<T>], &[T]>(uninit)
}

pub unsafe fn slice_as_init_mut<T>(uninit: &mut [MaybeUninit<T>]) -> &mut [T] {
    transmute::<&mut [MaybeUninit<T>], &mut [T]>(uninit)
}

pub fn copy_to_uninit_slice<T>(src: &[T], dst: &mut [MaybeUninit<T>]) {
    let len = src.len();
    assert_eq!(len, dst.len(), "buffer lengths do not match");

    let dst_ptr = dst.as_mut_ptr().cast();
    unsafe {
        src.as_ptr().copy_to_nonoverlapping(dst_ptr, len);
    }
}

impl<T: Sample, const N: usize> Default for DefaultBuf<T, N> {
    fn default() -> Self {
        let buf = <[T; N]>::silence();
        Self { buf }
    }
}

impl<T, const N: usize> Default for DefaultBuf<MaybeUninit<T>, N> {
    fn default() -> Self {
        let buf = <[T; N]>::new_uninit();
        Self { buf }
    }
}

impl<T, const N: usize> Deref for DefaultBuf<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl<T, const N: usize> DerefMut for DefaultBuf<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
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

impl<T, const N: usize> OwnedBuf for DefaultBuf<T, N> {
    type Item = T;
    type Uninit = [MaybeUninit<T>; N];

    unsafe fn from_uninit(uninit: Self::Uninit) -> Self {
        let buf = <_ as OwnedBuf>::from_uninit(uninit);
        Self { buf }
    }

    fn _as_slice(&self) -> &[T] {
        &self.buf
    }

    fn _as_mut_slice(&mut self) -> &mut [T] {
        &mut self.buf
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
    type Uninit = Rc<[MaybeUninit<T>]>;

    unsafe fn from_uninit(uninit: Self::Uninit) -> Self {
        transmute(uninit)
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
    type Uninit = Arc<[MaybeUninit<T>]>;

    unsafe fn from_uninit(uninit: Self::Uninit) -> Self {
        transmute(uninit)
    }

    fn _as_slice(&self) -> &[T] {
        self.deref()
    }

    fn _as_mut_slice(&mut self) -> &mut [T] {
        Arc::get_mut(self).unwrap_or(&mut [])
    }
}

impl<S: Sized, const N: usize> SizedBuf for [S; N] {
    fn new_uninit() -> Self::Uninit {
        unsafe { MaybeUninit::uninit().assume_init() }
    }
}

impl<S: Sample, const N: usize> SizedBuf for DefaultBuf<S, N> {
    fn new_uninit() -> Self::Uninit {
        <[S; N]>::new_uninit()
    }
}

impl<S: Sample> DynamicBuf for Vec<S> {
    fn new_uninit(capacity: usize) -> Self::Uninit {
        let mut buf = Vec::with_capacity(capacity);
        unsafe { buf.set_len(capacity) }

        buf
    }
}

impl<S: Sample> DynamicBuf for Box<[S]> {
    fn new_uninit(capacity: usize) -> Self::Uninit {
        Vec::new_uninit(capacity)
    }
}

impl<S: Sample> DynamicBuf for Rc<[S]> {
    fn new_uninit(len: usize) -> Self::Uninit {
        Rc::new_uninit_slice(len)
    }
}

impl<S: Sample> DynamicBuf for Arc<[S]> {
    fn new_uninit(len: usize) -> Self::Uninit {
        Arc::new_uninit_slice(len)
    }
}

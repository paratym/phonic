use crate::{
    utils::PollSignalReader, BlockingSignalReader, BufferedSignal, BufferedSignalReader,
    BufferedSignalWriter, FiniteSignal, IndexedSignal, PhonicError, PhonicResult, Sample, Signal,
    SignalReader, SignalSeeker, SignalSpec, SignalWriter,
};
use std::{
    mem::{transmute, MaybeUninit},
    ops::{Deref, DerefMut},
    rc::Rc,
    slice,
    sync::Arc,
    time::Duration,
};

pub const DEFAULT_BUF_LEN: usize = 4096;

pub struct DefaultBuf<S, const N: usize = DEFAULT_BUF_LEN> {
    pub buf: [S; N],
}

pub trait OwnedBuf<S>: Sized {
    type Uninitialized: OwnedBuf<MaybeUninit<S>>;
    unsafe fn from_uninit(uninit: Self::Uninitialized) -> Self;

    fn _as_slice(&self) -> &[S];
    fn _as_mut_slice(&mut self) -> &mut [S];
}

pub trait StaticBuf<S: Sized>: OwnedBuf<S> {
    fn new_uninit() -> Self::Uninitialized;

    fn silence() -> Self
    where
        S: Sample,
    {
        let mut buf = Self::new_uninit();
        buf._as_mut_slice().fill(MaybeUninit::new(S::ORIGIN));

        unsafe { Self::from_uninit(buf) }
    }

    fn read_exact_poll<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: PollSignalReader<Sample = S>,
    {
        let mut buf = Self::new_uninit();
        let uninit_slice = buf._as_mut_slice();
        let sample_slice = unsafe { transmute::<&mut [MaybeUninit<S>], &mut [S]>(uninit_slice) };
        reader.read_exact_poll(sample_slice)?;

        Ok(unsafe { Self::from_uninit(buf) })
    }

    fn read_exact_blocking<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: BlockingSignalReader<Sample = S>,
    {
        let mut buf = Self::new_uninit();
        let uninit_slice = buf._as_mut_slice();
        let sample_slice = unsafe { transmute::<&mut [MaybeUninit<S>], &mut [S]>(uninit_slice) };
        reader.read_exact_blocking(sample_slice)?;

        Ok(unsafe { Self::from_uninit(buf) })
    }
}

pub trait ResizeBuf<S>: DynamicBuf<S> {
    unsafe fn _resize(&mut self, len: usize);
}

pub trait DynamicBuf<S>: OwnedBuf<S> {
    fn new_uninit(capacity: usize) -> Self::Uninitialized;

    fn read_poll<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: PollSignalReader<Sample = S>,
        Self::Uninitialized: ResizeBuf<MaybeUninit<S>>,
    {
        let mut buf = Self::new_uninit(DEFAULT_BUF_LEN);
        let uninit_slice = buf._as_mut_slice();
        let sample_slice = unsafe { transmute::<&mut [MaybeUninit<S>], &mut [S]>(uninit_slice) };

        let n_samples = reader.read_poll(sample_slice)?;
        unsafe { buf._resize(n_samples) }

        Ok(unsafe { Self::from_uninit(buf) })
    }

    fn read_exact_poll<R>(reader: &mut R, n_frames: usize) -> PhonicResult<Self>
    where
        R: PollSignalReader<Sample = S>,
    {
        let n_samples = n_frames * reader.spec().channels.count() as usize;
        Self::read_exact_interleaved_poll(reader, n_samples)
    }

    fn read_exact_interleaved_poll<R>(reader: &mut R, n_samples: usize) -> PhonicResult<Self>
    where
        R: PollSignalReader<Sample = S>,
    {
        let mut buf = unsafe { Self::from_uninit(Self::new_uninit(n_samples)) };
        reader.read_exact_poll(buf._as_mut_slice())?;

        Ok(buf)
    }

    fn read_exact_duration_poll<R>(reader: &mut R, duration: Duration) -> PhonicResult<Self>
    where
        R: PollSignalReader<Sample = S>,
    {
        let n_frames = duration.as_secs_f64() * reader.spec().sample_rate as f64;
        Self::read_exact_poll(reader, n_frames as usize)
    }

    fn read_all_poll<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: PollSignalReader<Sample = S>,
        Self::Uninitialized: ResizeBuf<MaybeUninit<S>>,
    {
        let mut buf = Self::new_uninit(DEFAULT_BUF_LEN);
        let mut n_samples = 0;

        loop {
            let uninit_slice = buf._as_mut_slice();
            let sample_slice =
                unsafe { transmute::<&mut [MaybeUninit<S>], &mut [S]>(uninit_slice) };

            match reader.read_poll(sample_slice) {
                Ok(0) => break,
                Ok(n) => n_samples += n,
                Err(e) if n_samples == 0 => return Err(e),
                Err(e) => todo!(),
            }
        }

        unsafe { buf._resize(n_samples) }
        Ok(unsafe { Self::from_uninit(buf) })
    }

    fn read_blocking<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: BlockingSignalReader<Sample = S>,
        Self::Uninitialized: ResizeBuf<MaybeUninit<S>>,
    {
        let mut buf = Self::new_uninit(DEFAULT_BUF_LEN);
        let uninit_slice = buf._as_mut_slice();
        let sample_slice = unsafe { transmute::<&mut [MaybeUninit<S>], &mut [S]>(uninit_slice) };

        let n_samples = reader.read_blocking(sample_slice)?;
        unsafe { buf._resize(n_samples) }

        Ok(unsafe { Self::from_uninit(buf) })
    }

    fn read_exact_blocking<R>(reader: &mut R, n_frames: usize) -> PhonicResult<Self>
    where
        R: BlockingSignalReader<Sample = S>,
    {
        let n_samples = n_frames * reader.spec().channels.count() as usize;
        Self::read_exact_interleaved_blocking(reader, n_samples)
    }

    fn read_exact_interleaved_blocking<R>(reader: &mut R, n_samples: usize) -> PhonicResult<Self>
    where
        R: BlockingSignalReader<Sample = S>,
    {
        let mut buf = unsafe { Self::from_uninit(Self::new_uninit(n_samples)) };
        reader.read_exact_blocking(buf._as_mut_slice())?;
        Ok(buf)
    }

    fn read_exact_duration_blocking<R>(reader: &mut R, duration: Duration) -> PhonicResult<Self>
    where
        R: BlockingSignalReader<Sample = S>,
    {
        let n_frames = duration.as_secs_f64() * reader.spec().sample_rate as f64;
        Self::read_exact_blocking(reader, n_frames as usize)
    }

    fn read_all_blocking<R>(reader: &mut R) -> PhonicResult<Self>
    where
        R: BlockingSignalReader<Sample = S>,
    {
        todo!()
    }
}

impl<S: Sample, const N: usize> Default for DefaultBuf<S, N> {
    fn default() -> Self {
        let buf = <[S; N]>::silence();
        Self { buf }
    }
}

impl<S: Sample, const N: usize> Deref for DefaultBuf<S, N> {
    type Target = [S];

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl<S: Sample, const N: usize> DerefMut for DefaultBuf<S, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}

impl<S: Sized, const N: usize> OwnedBuf<S> for [S; N] {
    type Uninitialized = [MaybeUninit<S>; N];

    unsafe fn from_uninit(uninit: Self::Uninitialized) -> Self {
        let init_ptr = uninit.as_ptr().cast();
        unsafe { std::ptr::read(init_ptr) }
    }

    fn _as_slice(&self) -> &[S] {
        self.as_ref()
    }

    fn _as_mut_slice(&mut self) -> &mut [S] {
        self.as_mut()
    }
}

impl<S: Sized, const N: usize> OwnedBuf<S> for DefaultBuf<S, N> {
    type Uninitialized = [MaybeUninit<S>; N];

    unsafe fn from_uninit(uninit: Self::Uninitialized) -> Self {
        let buf = <_ as OwnedBuf<S>>::from_uninit(uninit);
        Self { buf }
    }

    fn _as_slice(&self) -> &[S] {
        &self.buf
    }

    fn _as_mut_slice(&mut self) -> &mut [S] {
        &mut self.buf
    }
}

impl<S: Sized> OwnedBuf<S> for Vec<S> {
    type Uninitialized = Vec<MaybeUninit<S>>;

    unsafe fn from_uninit(uninit: Self::Uninitialized) -> Self {
        transmute(uninit)
    }

    fn _as_slice(&self) -> &[S] {
        self.deref()
    }

    fn _as_mut_slice(&mut self) -> &mut [S] {
        self.deref_mut()
    }
}

impl<S: Sized> OwnedBuf<S> for Box<[S]> {
    type Uninitialized = Box<[MaybeUninit<S>]>;

    unsafe fn from_uninit(uninit: Self::Uninitialized) -> Self {
        transmute(uninit)
    }

    fn _as_slice(&self) -> &[S] {
        self.deref()
    }

    fn _as_mut_slice(&mut self) -> &mut [S] {
        self.deref_mut()
    }
}

impl<S: Sized> OwnedBuf<S> for Rc<[S]> {
    type Uninitialized = Rc<[MaybeUninit<S>]>;

    unsafe fn from_uninit(uninit: Self::Uninitialized) -> Self {
        transmute(uninit)
    }

    fn _as_slice(&self) -> &[S] {
        self.deref()
    }

    fn _as_mut_slice(&mut self) -> &mut [S] {
        Rc::get_mut(self).unwrap_or(&mut [])
    }
}

impl<S: Sized> OwnedBuf<S> for Arc<[S]> {
    type Uninitialized = Arc<[MaybeUninit<S>]>;

    unsafe fn from_uninit(uninit: Self::Uninitialized) -> Self {
        transmute(uninit)
    }

    fn _as_slice(&self) -> &[S] {
        self.deref()
    }

    fn _as_mut_slice(&mut self) -> &mut [S] {
        Arc::get_mut(self).unwrap_or(&mut [])
    }
}

impl<S: Sized, const N: usize> StaticBuf<S> for [S; N] {
    fn new_uninit() -> Self::Uninitialized {
        unsafe { MaybeUninit::uninit().assume_init() }
    }
}

impl<S: Sample, const N: usize> StaticBuf<S> for DefaultBuf<S, N> {
    fn new_uninit() -> Self::Uninitialized {
        <[S; N]>::new_uninit()
    }
}

impl<S: Sample> DynamicBuf<S> for Vec<S> {
    fn new_uninit(capacity: usize) -> Self::Uninitialized {
        let mut buf = Vec::with_capacity(capacity);
        unsafe { buf.set_len(capacity) }

        buf
    }
}

impl<S: Sample> DynamicBuf<S> for Box<[S]> {
    fn new_uninit(capacity: usize) -> Self::Uninitialized {
        Vec::new_uninit(capacity).into_boxed_slice()
    }
}

impl<S: Sample> DynamicBuf<S> for Rc<[S]> {
    fn new_uninit(capacity: usize) -> Self::Uninitialized {
        Rc::new_uninit_slice(capacity)
    }
}

impl<S: Sample> DynamicBuf<S> for Arc<[S]> {
    fn new_uninit(capacity: usize) -> Self::Uninitialized {
        Arc::new_uninit_slice(capacity)
    }
}

// pub struct BufSignal<B> {
//     spec: SignalSpec,
//     buf: B,
//     pos_interleaved: usize,
// }
//
// impl<B, S> Signal for BufSignal<B>
// where
//     B: Deref<Target = [S]>,
//     S: Sample,
// {
//     type Sample = S;
//
//     fn spec(&self) -> &SignalSpec {
//         &self.spec
//     }
// }
//
// impl<B, S> IndexedSignal for BufSignal<B>
// where
//     B: Deref<Target = [S]>,
//     S: Sample,
// {
//     fn pos(&self) -> u64 {
//         self.pos_interleaved as u64 / self.spec.channels.count() as u64
//     }
// }
//
// impl<B, S> FiniteSignal for BufSignal<B>
// where
//     B: Deref<Target = [S]>,
//     S: Sample,
// {
//     fn len(&self) -> u64 {
//         self.buf.len() as u64 / self.spec.channels.count() as u64
//     }
// }
//
// impl<B, S> SignalReader for BufSignal<B>
// where
//     B: Deref<Target = [S]>,
//     S: Sample,
// {
//     fn read(&mut self, buf: &mut [Self::Sample]) -> PhonicResult<usize> {
//         let n_samples = buf.len().min(self.buf.len() - self.pos_interleaved);
//         let inner_slice = &self.buf[self.pos_interleaved..self.pos_interleaved + n_samples];
//         buf[..n_samples].copy_from_slice(inner_slice);
//
//         self.pos_interleaved += n_samples;
//         Ok(n_samples)
//     }
// }
//
// impl<B, S> SignalWriter for BufSignal<B>
// where
//     B: DerefMut<Target = [S]>,
//     S: Sample,
// {
//     fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
//         let n_samples = buf.len().min(self.buf.len() - self.pos_interleaved);
//         let inner_slice = &mut self.buf[self.pos_interleaved..self.pos_interleaved + n_samples];
//         inner_slice.copy_from_slice(&buf[..n_samples]);
//
//         self.pos_interleaved += n_samples;
//         Ok(n_samples)
//     }
//
//     fn flush(&mut self) -> PhonicResult<()> {
//         Ok(())
//     }
// }
//
// impl<B, S> SignalSeeker for BufSignal<B>
// where
//     B: Deref<Target = [S]>,
//     S: Sample,
// {
//     fn seek(&mut self, offset: i64) -> PhonicResult<()> {
//         let pos = match self.pos().checked_add_signed(offset) {
//             None => return Err(PhonicError::OutOfBounds),
//             Some(pos) if pos > self.len() => return Err(PhonicError::OutOfBounds),
//             Some(pos) => pos,
//         };
//
//         let pos_interleaved = pos * self.spec.channels.count() as u64;
//         self.pos_interleaved = pos_interleaved as usize;
//
//         Ok(())
//     }
// }
//
// impl<B, S> BufferedSignal for BufSignal<B>
// where
//     B: Deref<Target = [S]>,
//     S: Sample,
// {
//     fn commit_samples(&mut self, n_samples: usize) {
//         debug_assert_eq!(n_samples % self.spec.channels.count() as usize, 0);
//         self.pos_interleaved += n_samples
//     }
// }
//
// impl<B, S> BufferedSignalReader for BufSignal<B>
// where
//     B: Deref<Target = [S]>,
//     S: Sample,
// {
//     fn available_samples(&self) -> &[Self::Sample] {
//         &self.buf[self.pos_interleaved..]
//     }
// }
//
// impl<B, S> BufferedSignalWriter for BufSignal<B>
// where
//     B: DerefMut<Target = [S]>,
//     S: Sample,
// {
//     fn available_slots(&mut self) -> &mut [MaybeUninit<Self::Sample>] {
//         todo!()
//     }
// }

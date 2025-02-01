use crate::{
    delegate_signal,
    utils::{
        copy_to_uninit_slice, slice_as_init, DefaultSizedBuf, DynamicBuf, IntoDuration, NSamples,
        SizedBuf,
    },
    BufferedSignalReader, BufferedSignalWriter, PhonicError, PhonicResult, Signal, SignalReader,
    SignalSeeker, SignalWriter,
};
use std::{borrow::BorrowMut, mem::MaybeUninit};

pub struct BufReader<T: Signal, B = DefaultSizedBuf<<T as Signal>::Sample>> {
    inner: T,
    buf: B,
    idx: usize,
    len: usize,
    exhausted: bool,
}

pub struct BufWriter<T: Signal, B = DefaultSizedBuf<<T as Signal>::Sample>> {
    inner: T,
    buf: B,
    idx: usize,
    len: usize,
    exhausted: bool,
}

impl<T: Signal, B> BufReader<T, B> {
    pub fn new(inner: T, buf: B) -> Self {
        Self {
            inner,
            buf,
            idx: 0,
            len: 0,
            exhausted: false,
        }
    }

    pub fn default<D>(inner: T, duration: D) -> BufReader<T, B::Uninit>
    where
        B: DynamicBuf,
        D: IntoDuration<NSamples>,
    {
        let NSamples { n_samples } = duration.into_duration(inner.spec());
        let buf = B::uninit(n_samples as usize);

        BufReader::new(inner, buf)
    }

    pub fn default_sized(inner: T) -> BufReader<T, B::Uninit>
    where
        B: SizedBuf,
    {
        BufReader::new(inner, B::uninit())
    }
}

impl<T: Signal, B> BufWriter<T, B> {
    pub fn new(inner: T, buf: B) -> Self {
        Self {
            inner,
            buf,
            idx: 0,
            len: 0,
            exhausted: false,
        }
    }

    pub fn default<D>(inner: T, duration: D) -> BufReader<T, B::Uninit>
    where
        B: DynamicBuf,
        D: IntoDuration<NSamples>,
    {
        let NSamples { n_samples } = duration.into_duration(inner.spec());
        let buf = B::uninit(n_samples as usize);

        BufReader::new(inner, buf)
    }

    pub fn default_sized(inner: T) -> BufReader<T, B::Uninit>
    where
        B: SizedBuf,
    {
        BufReader::new(inner, B::uninit())
    }
}

delegate_signal! {
    impl<T: Signal, B> * + !Mut for BufReader<T, B> {
        Self as T;

        &self => &self.inner;
    }
}

delegate_signal! {
    impl<T: Signal, B> * + !Mut for BufWriter<T, B> {
        Self as T;

        &self => &self.inner;
    }
}

impl<T, B> SignalReader for BufReader<T, B>
where
    T: SignalReader,
    B: BorrowMut<[MaybeUninit<T::Sample>]>,
{
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        let n_channels = self.spec().n_channels;
        let samples = self.fill()?;

        let mut len = buf.len().min(samples.len());
        len -= len % n_channels;

        copy_to_uninit_slice(&samples[..len], &mut buf[..len]);
        self.consume(len);

        Ok(len)
    }
}

impl<T, B> BufferedSignalReader for BufReader<T, B>
where
    T: SignalReader,
    B: BorrowMut<[MaybeUninit<T::Sample>]>,
{
    fn fill(&mut self) -> PhonicResult<&[Self::Sample]> {
        // if self.exhausted {
        //     let buf = self.buf.borrow_mut();
        //     let slice = &buf[..0];
        //     let samples = unsafe { slice_as_init(slice) };
        //
        //     return Ok(samples);
        // }

        if self.idx < self.len {
            let buf = self.buf.borrow_mut();
            let slice = &buf[self.idx..self.len];
            let samples = unsafe { slice_as_init(slice) };

            return Ok(samples);
        }

        debug_assert_eq!(self.idx, self.len);

        let buf = self.buf.borrow_mut();
        self.len = self.inner.read(buf)?;
        self.exhausted = self.len == 0;

        let slice = &buf[..self.len];
        let samples = unsafe { slice_as_init(slice) };

        Ok(samples)
    }

    fn buffer(&self) -> Option<&[Self::Sample]> {
        if self.exhausted {
            return None;
        }

        let buf = self.buf.borrow();
        let slice = &buf[self.idx..self.len];
        let samples = unsafe { slice_as_init(slice) };

        Some(samples)
    }

    fn consume(&mut self, n_samples: usize) {
        assert!(n_samples <= self.len - self.idx);
        debug_assert!(n_samples % self.spec().n_channels == 0);

        self.idx += n_samples;
        if self.idx == self.len {
            self.idx = 0;
            self.len = 0;
        }
    }
}

impl<T, B> SignalWriter for BufWriter<T, B>
where
    T: SignalWriter,
    B: BorrowMut<[MaybeUninit<T::Sample>]>,
{
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        let n_channels = self.spec().n_channels;
        let mut cap = self.buf.borrow().len();
        cap -= cap % n_channels;

        if self.len == cap {
            self.flush()?;
        }

        let idx = self.idx;
        let Some(inner_buf) = self.buffer_mut() else {
            return Ok(0);
        };

        let mut len = inner_buf.len().min(buf.len());
        len -= n_channels;

        copy_to_uninit_slice(&buf[..len], &mut inner_buf[idx..idx + len]);
        Ok(len)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        let buf = self.buf.borrow();
        while self.idx < self.len {
            let slice = &buf[self.idx..self.len];
            let samples = unsafe { slice_as_init(slice) };

            let n_samples = self.inner.write(samples)?;
            self.idx += n_samples;

            if n_samples == 0 {
                self.exhausted = true;
                return Err(PhonicError::out_of_bounds());
            }
        }

        Ok(())
    }
}

impl<T, B> BufferedSignalWriter for BufWriter<T, B>
where
    T: SignalWriter,
    B: BorrowMut<[MaybeUninit<T::Sample>]>,
{
    fn buffer_mut(&mut self) -> Option<&mut [MaybeUninit<Self::Sample>]> {
        if self.exhausted {
            return None;
        }

        let n_channels = self.spec().n_channels;
        let buf = self.buf.borrow_mut();

        let mut n_slots = buf.len() - self.len;
        n_slots -= n_slots % n_channels;

        let slots = &mut buf[self.len..self.len + n_slots];
        Some(slots)
    }

    fn commit(&mut self, n_samples: usize) {
        let cap = self.buf.borrow().len();
        assert!(n_samples <= cap - self.len);
        debug_assert!(n_samples % self.spec().n_channels == 0);

        self.len += n_samples;
    }
}

impl<T: SignalSeeker, B> SignalSeeker for BufReader<T, B> {
    fn seek(&mut self, n_frames: i64) -> PhonicResult<()> {
        todo!()
    }
}

impl<T: SignalSeeker, B> SignalSeeker for BufWriter<T, B> {
    fn seek(&mut self, n_frames: i64) -> PhonicResult<()> {
        todo!()
    }
}

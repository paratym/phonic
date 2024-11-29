use phonic_signal::{PhonicError, Sample, Signal, SignalReader, SignalSpec, SignalWriter};
use rtrb::{chunks::ChunkError, Consumer, CopyToUninit, Producer, RingBuffer};
use std::{marker::PhantomData, time::Duration};

pub struct SignalBuffer;
pub struct SignalBufferHalf<T, S> {
    spec: SignalSpec,
    inner: T,
    _sample: PhantomData<S>,
}

pub type SignalBufferPair<S> = (
    SignalBufferHalf<Producer<S>, S>,
    SignalBufferHalf<Consumer<S>, S>,
);

impl SignalBuffer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<S>(spec: SignalSpec, n_frames: u64) -> SignalBufferPair<S> {
        let n_samples = spec.channels.count() as usize * n_frames as usize;
        Self::new_interleaved(spec, n_samples)
    }

    pub fn new_interleaved<S>(spec: SignalSpec, n_samples: usize) -> SignalBufferPair<S> {
        let (producer, consumer) = RingBuffer::new(n_samples);

        (
            SignalBufferHalf::new(producer, spec),
            SignalBufferHalf::new(consumer, spec),
        )
    }

    pub fn new_duration<S>(spec: SignalSpec, duration: Duration) -> SignalBufferPair<S> {
        let n_samples = duration.as_secs_f64() * spec.sample_rate_interleaved() as f64;
        Self::new_interleaved(spec, n_samples as usize)
    }
}

impl<T, S> SignalBufferHalf<T, S> {
    fn new(inner: T, spec: SignalSpec) -> Self {
        Self {
            inner,
            spec,
            _sample: PhantomData,
        }
    }
}

impl<T, S: Sample> Signal for SignalBufferHalf<T, S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<S: Sample> SignalBufferHalf<Consumer<S>, S> {
    fn _read(&mut self, buf: &mut [S], exact: bool) -> Result<usize, PhonicError> {
        let n_slots = self.inner.slots();
        let buf_len = buf.len();

        if exact && n_slots > buf_len {
            return if self.inner.is_abandoned() {
                Err(PhonicError::OutOfBounds)
            } else {
                Err(PhonicError::NotReady)
            };
        }

        if n_slots == 0 {
            return if self.inner.is_abandoned() {
                Ok(0)
            } else {
                Err(PhonicError::NotReady)
            };
        }

        let n_samples = buf_len.min(n_slots);
        let read_chunk = self.inner.read_chunk(n_samples).unwrap();

        let (head, tail) = read_chunk.as_slices();
        let head_len = head.len();
        let tail_end = head_len + tail.len();

        buf[..head_len].copy_from_slice(head);
        buf[head_len..tail_end].copy_from_slice(tail);

        Ok(n_samples)
    }
}

impl<S: Sample> SignalReader for SignalBufferHalf<Consumer<S>, S> {
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        self._read(buf, false)
    }

    // fn read_exact(&mut self, buf: &mut [Self::Sample], block: bool) -> Result<(), PhonicError> {
    //     let buf_len = buf.len();
    //     if buf_len % self.spec.channels.count() as usize != 0 {
    //         return Err(PhonicError::SignalMismatch);
    //     }
    //
    //     loop {
    //         match self._read(buf, true) {
    //             Ok(n) => {
    //                 assert!(n == buf_len);
    //                 return Ok(());
    //             }
    //             Err(PhonicError::Interrupted) if block => continue,
    //             Err(PhonicError::NotReady) if block => continue, // sleep
    //             Err(e) => return Err(e),
    //         }
    //     }
    // }
}

impl<S: Sample> SignalBufferHalf<Producer<S>, S> {
    fn _write(&mut self, buf: &[S], exact: bool) -> Result<usize, PhonicError> {
        let n_slots = self.inner.slots();
        let buf_len = buf.len();

        if exact && n_slots < buf_len {
            return if self.inner.is_abandoned() {
                Err(PhonicError::OutOfBounds)
            } else {
                Err(PhonicError::NotReady)
            };
        }

        if n_slots == 0 {
            return if self.inner.is_abandoned() {
                Ok(0)
            } else {
                Err(PhonicError::NotReady)
            };
        }

        let n_samples = buf_len.min(n_slots);
        if exact && n_samples < buf_len {
            return Err(PhonicError::NotReady);
        }

        let n_samples = buf.len().min(n_slots);
        let mut write_chunk = self.inner.write_chunk_uninit(n_samples).unwrap();

        let (head, tail) = write_chunk.as_mut_slices();
        let head_len = head.len();
        let tail_end = head_len + tail.len();

        buf[..head_len].copy_to_uninit(head);
        buf[head_len..tail_end].copy_to_uninit(tail);

        Ok(n_samples)
    }
}

impl<S: Sample> SignalWriter for SignalBufferHalf<Producer<S>, S> {
    fn write(&mut self, buf: &[Self::Sample]) -> Result<usize, PhonicError> {
        self._write(buf, false)
    }

    fn flush(&mut self) -> Result<(), PhonicError> {
        todo!()
    }

    // TODO: optimize copy functions
}

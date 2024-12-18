use phonic_signal::{
    utils::DEFAULT_BUF_LEN, BlockingSignalReader, BlockingSignalWriter, BufferedSignal,
    BufferedSignalReader, BufferedSignalWriter, PhonicError, PhonicResult, Sample, Signal,
    SignalReader, SignalSpec, SignalWriter,
};
use rtrb::{Consumer, CopyToUninit, Producer, RingBuffer};
use std::{mem::MaybeUninit, time::Duration};

pub struct SignalBuffer;

pub struct SignalProducer<S> {
    spec: SignalSpec,
    inner: Producer<S>,
}

pub struct SignalConsumer<S> {
    spec: SignalSpec,
    inner: Consumer<S>,
}

pub type SignalBufferPair<S> = (SignalProducer<S>, SignalConsumer<S>);

impl SignalBuffer {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<S>(spec: SignalSpec, n_frames: u64) -> SignalBufferPair<S> {
        let n_samples = spec.channels.count() as usize * n_frames as usize;
        Self::new_interleaved(spec, n_samples)
    }

    pub fn new_interleaved<S>(spec: SignalSpec, n_samples: usize) -> SignalBufferPair<S> {
        let (producer, consumer) = RingBuffer::new(n_samples);

        (
            SignalProducer {
                spec,
                inner: producer,
            },
            SignalConsumer {
                spec,
                inner: consumer,
            },
        )
    }

    pub fn new_duration<S>(spec: SignalSpec, duration: Duration) -> SignalBufferPair<S> {
        let n_samples = duration.as_secs_f64() * spec.sample_rate_interleaved() as f64;
        Self::new_interleaved(spec, n_samples as usize)
    }

    pub fn default<S>(spec: SignalSpec) -> SignalBufferPair<S> {
        Self::new_interleaved(spec, DEFAULT_BUF_LEN)
    }
}

impl<S: Sample> Signal for SignalConsumer<S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<S: Sample> BufferedSignal for SignalConsumer<S> {
    fn commit_samples(&mut self, n_samples: usize) {
        todo!()
    }
}

impl<S: Sample> SignalConsumer<S> {
    fn _read(
        &mut self,
        buf: &mut [MaybeUninit<<Self as Signal>::Sample>],
        blocking: bool,
    ) -> PhonicResult<usize> {
        let n_slots = self.inner.slots();
        let buf_len = match n_slots.min(buf.len()) {
            0 if buf.is_empty() => return Err(PhonicError::InvalidInput),
            0 if self.inner.is_abandoned() => return Ok(0),
            0 if blocking => todo!(),
            0 => return Err(PhonicError::NotReady),
            n => n,
        };

        let read_chunk = self.inner.read_chunk(buf_len).unwrap();
        let (head, tail) = read_chunk.as_slices();

        let head_len = head.len();
        let tail_end = head_len + tail.len();

        head.copy_to_uninit(&mut buf[..head_len]);
        tail.copy_to_uninit(&mut buf[head_len..tail_end]);

        Ok(buf_len)
    }
}

impl<S: Sample> SignalReader for SignalConsumer<S> {
    fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        self._read(buf, false)
    }
}

impl<S: Sample> BlockingSignalReader for SignalConsumer<S> {
    fn read_blocking(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
        self._read(buf, true)
    }
}

impl<S: Sample> BufferedSignalReader for SignalConsumer<S> {
    fn available_samples(&self) -> &[Self::Sample] {
        todo!()
    }
}

impl<S: Sample> Signal for SignalProducer<S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<S: Sample> BufferedSignal for SignalProducer<S> {
    fn commit_samples(&mut self, n_samples: usize) {
        todo!()
    }
}

impl<S: Sample> SignalProducer<S> {
    fn _write(&mut self, buf: &[<Self as Signal>::Sample], blocking: bool) -> PhonicResult<usize> {
        if self.inner.is_abandoned() {
            return Ok(0);
        }

        let n_slots = self.inner.slots();
        let buf_len = match n_slots.min(buf.len()) {
            0 if buf.is_empty() => return Err(PhonicError::InvalidInput),
            0 if blocking => todo!(),
            0 => return Err(PhonicError::NotReady),
            n => n,
        };

        let mut write_chunk = self.inner.write_chunk_uninit(buf_len).unwrap();
        let (head, tail) = write_chunk.as_mut_slices();

        let head_len = head.len();
        let tail_end = head_len + tail.len();
        buf[..head_len].copy_to_uninit(head);
        buf[head_len..tail_end].copy_to_uninit(tail);

        Ok(buf_len)
    }
}

impl<S: Sample> SignalWriter for SignalProducer<S> {
    fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        self._write(buf, false)
    }

    fn flush(&mut self) -> PhonicResult<()> {
        todo!()
    }
}

impl<S: Sample> BlockingSignalWriter for SignalProducer<S> {
    fn write_blocking(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
        self._write(buf, true)
    }

    fn flush_blocking(&mut self) -> PhonicResult<()> {
        todo!()
    }
}

impl<S: Sample> BufferedSignalWriter for SignalProducer<S> {
    fn available_slots(&mut self) -> &mut [MaybeUninit<Self::Sample>] {
        todo!()
    }
}

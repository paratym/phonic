use cpal::{
    traits::DeviceTrait, BufferSize, BuildStreamError, InputCallbackInfo, OutputCallbackInfo,
    SampleFormat, SampleRate, SizedSample, StreamConfig, StreamError, SupportedStreamConfigRange,
};
use phonic_signal::{
    utils::slice_as_uninit_mut, PhonicError, Sample, Signal, SignalReader, SignalSpec, SignalWriter,
};
use std::{
    any::TypeId,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

pub trait SignalSpecExt {
    fn from_cpal_config(config: StreamConfig) -> Self;
    fn into_cpal_config(self, buffer_size: BufferSize) -> StreamConfig;
}

impl SignalSpecExt for SignalSpec {
    fn from_cpal_config(config: StreamConfig) -> SignalSpec {
        SignalSpec {
            sample_rate: config.sample_rate.0,
            channels: (config.channels as u32).into(),
        }
    }

    fn into_cpal_config(self, buffer_size: BufferSize) -> StreamConfig {
        StreamConfig {
            channels: self.channels.count() as u16,
            sample_rate: SampleRate(self.sample_rate),
            buffer_size,
        }
    }
}

pub trait SupportedStreamConfigRangeExt {
    fn supports_signal<S: Signal>(&self, signal: &S) -> bool;
}

impl SupportedStreamConfigRangeExt for SupportedStreamConfigRange {
    fn supports_signal<S: Signal>(&self, signal: &S) -> bool {
        let spec = signal.spec();
        let sample_type = match self.sample_format() {
            SampleFormat::I8 => TypeId::of::<i8>(),
            SampleFormat::I16 => TypeId::of::<i16>(),
            SampleFormat::I32 => TypeId::of::<i32>(),
            SampleFormat::I64 => TypeId::of::<i64>(),
            SampleFormat::U8 => TypeId::of::<u8>(),
            SampleFormat::U16 => TypeId::of::<u16>(),
            SampleFormat::U32 => TypeId::of::<u32>(),
            SampleFormat::U64 => TypeId::of::<u64>(),
            SampleFormat::F32 => TypeId::of::<f32>(),
            SampleFormat::F64 => TypeId::of::<f64>(),
            _ => return false,
        };

        sample_type == TypeId::of::<S::Sample>()
            && self.channels() == spec.channels.count() as u16
            && self.max_sample_rate().0 >= spec.sample_rate
            && self.min_sample_rate().0 <= spec.sample_rate
    }
}

pub struct CpalSignal<Exhausted = fn(), SignalErr = fn(PhonicError), CpalErr = fn(StreamError)>
where
    Exhausted: FnOnce() + Send + 'static,
    SignalErr: FnOnce(PhonicError) + Send + 'static,
    CpalErr: FnMut(StreamError) + Send + 'static,
{
    pub buffer_size: BufferSize,
    pub timeout: Option<Duration>,
    pub on_exhausted: Option<Exhausted>,
    pub on_signal_err: Option<SignalErr>,
    pub on_cpal_err: Option<CpalErr>,
}

impl<Exhausted, SignalErr, CpalErr> CpalSignal<Exhausted, SignalErr, CpalErr>
where
    Exhausted: FnOnce() + Send + 'static,
    SignalErr: FnOnce(PhonicError) + Send + 'static,
    CpalErr: FnMut(StreamError) + Send + 'static,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn buffer_size(mut self, buffer_size: u32) -> Self {
        self.buffer_size = BufferSize::Fixed(buffer_size);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn on_exhausted<F>(self, callback: F) -> CpalSignal<F, SignalErr, CpalErr>
    where
        F: FnOnce() + Send + 'static,
    {
        CpalSignal {
            buffer_size: self.buffer_size,
            timeout: self.timeout,
            on_exhausted: Some(callback),
            on_signal_err: self.on_signal_err,
            on_cpal_err: self.on_cpal_err,
        }
    }

    pub fn on_signal_err<F>(self, callback: F) -> CpalSignal<Exhausted, F, CpalErr>
    where
        F: FnMut(PhonicError) + Send + 'static,
    {
        CpalSignal {
            buffer_size: self.buffer_size,
            timeout: self.timeout,
            on_exhausted: self.on_exhausted,
            on_signal_err: Some(callback),
            on_cpal_err: self.on_cpal_err,
        }
    }

    pub fn on_cpal_err<F>(self, callback: F) -> CpalSignal<Exhausted, SignalErr, F>
    where
        F: FnMut(StreamError) + Send + 'static,
    {
        CpalSignal {
            buffer_size: self.buffer_size,
            timeout: self.timeout,
            on_exhausted: self.on_exhausted,
            on_signal_err: self.on_signal_err,
            on_cpal_err: Some(callback),
        }
    }

    pub fn build_input<D, S>(self, device: &D, mut signal: S) -> Result<D::Stream, BuildStreamError>
    where
        D: DeviceTrait,
        S: SignalWriter + Send + 'static,
        S::Sample: SizedSample,
    {
        let Self {
            buffer_size,
            timeout,
            mut on_exhausted,
            mut on_signal_err,
            mut on_cpal_err,
        } = self;

        let config = signal.spec().into_cpal_config(buffer_size);

        let exited = AtomicBool::default();
        let data_callback = move |buf: &[S::Sample], _: &InputCallbackInfo| {
            if exited.load(Ordering::Relaxed) {
                return;
            }

            match signal.write(buf) {
                Ok(0) => {
                    exited.store(true, Ordering::Relaxed);
                    if let Some(callback) = on_exhausted.take() {
                        callback()
                    }
                }

                Ok(_) => {}
                Err(PhonicError::Interrupted { .. }) => {}

                Err(err) => {
                    exited.store(true, Ordering::Relaxed);
                    if let Some(callback) = on_signal_err.take() {
                        callback(err);
                    }
                }
            }
        };

        let error_callback = move |err: StreamError| {
            if let Some(ref mut callback) = on_cpal_err {
                callback(err)
            }
        };

        device.build_input_stream(&config, data_callback, error_callback, timeout)
    }

    pub fn build_output<D, S>(
        self,
        device: &D,
        mut signal: S,
    ) -> Result<D::Stream, BuildStreamError>
    where
        D: DeviceTrait,
        S: SignalReader + Send + 'static,
        S::Sample: SizedSample,
    {
        let Self {
            buffer_size,
            timeout,
            mut on_exhausted,
            mut on_signal_err,
            mut on_cpal_err,
        } = self;

        let config = signal.spec().into_cpal_config(buffer_size);

        let exited = AtomicBool::default();
        let data_callback = move |buf: &mut [S::Sample], _: &OutputCallbackInfo| {
            if exited.load(Ordering::Relaxed) {
                buf.fill(S::Sample::ORIGIN);
                return;
            }

            let uninit_buf = slice_as_uninit_mut(buf);
            match signal.read(uninit_buf) {
                Ok(0) => {
                    exited.store(true, Ordering::Relaxed);
                    if let Some(callback) = on_exhausted.take() {
                        callback()
                    }
                }

                Ok(n) => buf[n..].fill(S::Sample::ORIGIN),
                Err(PhonicError::Interrupted { .. }) => buf.fill(S::Sample::ORIGIN),

                Err(err) => {
                    exited.store(true, Ordering::Relaxed);
                    if let Some(callback) = on_signal_err.take() {
                        callback(err)
                    }
                }
            }
        };

        let error_callback = move |err: StreamError| {
            if let Some(ref mut callback) = on_cpal_err {
                callback(err)
            }
        };

        device.build_output_stream(&config, data_callback, error_callback, timeout)
    }
}

impl<Exhausted, SignalErr, CpalErr> Default for CpalSignal<Exhausted, SignalErr, CpalErr>
where
    Exhausted: FnOnce() + Send + 'static,
    SignalErr: FnOnce(PhonicError) + Send + 'static,
    CpalErr: FnMut(StreamError) + Send + 'static,
{
    fn default() -> Self {
        Self {
            buffer_size: BufferSize::Default,
            timeout: None,
            on_exhausted: None,
            on_signal_err: None,
            on_cpal_err: None,
        }
    }
}

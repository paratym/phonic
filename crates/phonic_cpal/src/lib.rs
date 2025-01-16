use cpal::{
    traits::DeviceTrait, BufferSize, BuildStreamError, InputCallbackInfo, OutputCallbackInfo,
    SampleFormat, SampleRate, SizedSample, StreamConfig, StreamError, SupportedStreamConfigRange,
};
use phonic_signal::{
    utils::copy_to_uninit_slice, BufferedSignalReader, BufferedSignalWriter, Sample, Signal,
    SignalSpec,
};
use std::{any::TypeId, time::Duration};

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

pub trait DeviceExt: DeviceTrait {
    fn build_input_stream_from_signal<T, E>(
        &self,
        mut signal: T,
        error_callback: E,
        buffer_size: BufferSize,
        timeout: Option<Duration>,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        T: BufferedSignalWriter + Send + 'static,
        T::Sample: SizedSample,
        E: FnMut(StreamError) + Send + 'static,
    {
        self.build_input_stream(
            &signal.spec().into_cpal_config(buffer_size),
            move |buf: &[T::Sample], _: &InputCallbackInfo| {
                let inner_buf = signal.buffer_mut().unwrap_or_default();
                let n_samples = inner_buf.len().min(buf.len());
                debug_assert!(buf.len() <= inner_buf.len());

                copy_to_uninit_slice(&buf[..n_samples], &mut inner_buf[..n_samples]);
                signal.commit(n_samples);
            },
            error_callback,
            timeout,
        )
    }

    fn build_output_stream_from_signal<T, E>(
        &self,
        mut signal: T,
        error_callback: E,
        buffer_size: BufferSize,
        timeout: Option<Duration>,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        T: BufferedSignalReader + Send + 'static,
        T::Sample: SizedSample,
        E: FnMut(StreamError) + Send + 'static,
    {
        self.build_output_stream(
            &signal.spec().into_cpal_config(buffer_size),
            move |buf: &mut [T::Sample], _: &OutputCallbackInfo| {
                let inner_buf = signal.buffer().unwrap_or_default();
                let n_samples = inner_buf.len().min(buf.len());
                debug_assert!(buf.len() <= inner_buf.len());

                buf[..n_samples].copy_from_slice(&inner_buf[..n_samples]);
                signal.consume(n_samples);

                buf[n_samples..].fill(T::Sample::ORIGIN);
            },
            error_callback,
            timeout,
        )
    }
}

impl<T: DeviceTrait> DeviceExt for T {}

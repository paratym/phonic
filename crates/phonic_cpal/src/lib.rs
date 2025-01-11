use cpal::{
    traits::DeviceTrait, BufferSize, BuildStreamError, InputCallbackInfo, OutputCallbackInfo,
    SampleFormat, SampleRate, SizedSample, StreamConfig, StreamError, SupportedStreamConfigRange,
};
use phonic_io_core::{KnownSample, KnownSampleType};
use phonic_signal::{
    utils::copy_to_uninit_slice, BufferedSignalReader, BufferedSignalWriter, PhonicError,
    PhonicResult, Signal, SignalSpec,
};
use std::time::Duration;

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

pub trait KnownSampleTypeExt: Sized {
    fn try_from_cpal_sample_format(format: SampleFormat) -> PhonicResult<Self>;
    fn into_cpal_sample_format(self) -> SampleFormat;
}

impl KnownSampleTypeExt for KnownSampleType {
    fn try_from_cpal_sample_format(format: SampleFormat) -> PhonicResult<Self> {
        Ok(match format {
            SampleFormat::I8 => Self::I8,
            SampleFormat::I16 => Self::I16,
            SampleFormat::I32 => Self::I32,
            SampleFormat::I64 => Self::I64,
            SampleFormat::U8 => Self::U8,
            SampleFormat::U16 => Self::U16,
            SampleFormat::U32 => Self::U32,
            SampleFormat::U64 => Self::U64,
            SampleFormat::F32 => Self::F32,
            SampleFormat::F64 => Self::F64,
            _ => return Err(PhonicError::Unsupported),
        })
    }

    fn into_cpal_sample_format(self) -> SampleFormat {
        match self {
            Self::I8 => SampleFormat::I8,
            Self::I16 => SampleFormat::I16,
            Self::I32 => SampleFormat::I32,
            Self::I64 => SampleFormat::I64,
            Self::U8 => SampleFormat::U8,
            Self::U16 => SampleFormat::U16,
            Self::U32 => SampleFormat::U32,
            Self::U64 => SampleFormat::U64,
            Self::F32 => SampleFormat::F32,
            Self::F64 => SampleFormat::F64,
        }
    }
}

pub trait SupportedStreamConfigRangeExt {
    fn supports_signal<S>(&self, signal: &S) -> bool
    where
        S: Signal,
        S::Sample: KnownSample;
}

impl SupportedStreamConfigRangeExt for SupportedStreamConfigRange {
    fn supports_signal<S>(&self, signal: &S) -> bool
    where
        S: Signal,
        S::Sample: KnownSample,
    {
        let spec = signal.spec();

        self.sample_format() == S::Sample::TYPE.into_cpal_sample_format()
            && self.channels() == spec.channels.count() as u16
            && self.max_sample_rate().0 >= spec.sample_rate
            && self.min_sample_rate().0 <= spec.sample_rate
    }
}

pub trait DeviceExt: DeviceTrait {
    fn build_input_stream_from_signal<S, E>(
        &self,
        mut signal: S,
        error_callback: E,
        buffer_size: BufferSize,
        timeout: Option<Duration>,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        S: BufferedSignalWriter + Send + 'static,
        S::Sample: SizedSample + KnownSample,
        E: FnMut(StreamError) + Send + 'static,
    {
        self.build_input_stream(
            &signal.spec().into_cpal_config(buffer_size),
            move |buf: &[S::Sample], _: &InputCallbackInfo| {
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

    fn build_output_stream_from_signal<S, E>(
        &self,
        mut signal: S,
        error_callback: E,
        buffer_size: BufferSize,
        timeout: Option<Duration>,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        S: BufferedSignalReader + Send + 'static,
        S::Sample: SizedSample + KnownSample,
        E: FnMut(StreamError) + Send + 'static,
    {
        self.build_output_stream(
            &signal.spec().into_cpal_config(buffer_size),
            move |buf: &mut [S::Sample], _: &OutputCallbackInfo| {
                let inner_buf = signal.buffer().unwrap_or_default();
                let n_samples = inner_buf.len().min(buf.len());
                debug_assert!(buf.len() <= inner_buf.len());

                buf[..n_samples].copy_from_slice(&inner_buf[..n_samples]);
                signal.consume(n_samples);
            },
            error_callback,
            timeout,
        )
    }
}

impl<T: DeviceTrait> DeviceExt for T {}

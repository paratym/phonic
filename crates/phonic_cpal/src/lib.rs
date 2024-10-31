use cpal::{
    traits::DeviceTrait, BufferSize, BuildStreamError, InputCallbackInfo, OutputCallbackInfo,
    SampleFormat, SampleRate, SizedSample, StreamConfig, StreamError, SupportedStreamConfigRange,
};
use phonic_core::PhonicError;
use phonic_io_core::{KnownSample, KnownSampleType};
use phonic_signal::{Sample, Signal, SignalReader, SignalSpec, SignalWriter};
use std::time::Duration;

pub trait SignalSpecExt {
    fn from_cpal_config(config: StreamConfig) -> Self;
    fn into_cpal_config(self) -> StreamConfig;
}

impl SignalSpecExt for SignalSpec {
    fn from_cpal_config(config: StreamConfig) -> SignalSpec {
        SignalSpec {
            sample_rate: config.sample_rate.0,
            channels: (config.channels as u32).into(),
        }
    }

    fn into_cpal_config(self) -> StreamConfig {
        StreamConfig {
            channels: self.channels.count() as u16,
            sample_rate: SampleRate(self.sample_rate),
            buffer_size: BufferSize::Default,
        }
    }
}

pub trait KnownSampleTypeExt: Sized {
    fn try_from_cpal_sample_format(format: SampleFormat) -> Result<Self, PhonicError>;
    fn into_cpal_sample_format(self) -> SampleFormat;
}

impl KnownSampleTypeExt for KnownSampleType {
    fn try_from_cpal_sample_format(format: SampleFormat) -> Result<Self, PhonicError> {
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
        timeout: Option<Duration>,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        S: SignalWriter + Send + 'static,
        S::Sample: SizedSample + KnownSample,
        E: FnMut(StreamError) + Send + 'static,
    {
        self.build_input_stream(
            &signal.spec().into_cpal_config(),
            move |buf: &[S::Sample], _: &InputCallbackInfo| match signal.write_exact(buf, false) {
                Ok(_) | Err(PhonicError::NotReady) | Err(PhonicError::Interrupted) => {}
                Err(e) => panic!("error writing to signal: {e}"),
            },
            error_callback,
            timeout,
        )
    }

    fn build_output_stream_from_signal<S, E>(
        &self,
        mut signal: S,
        error_callback: E,
        timeout: Option<Duration>,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        S: SignalReader + Send + 'static,
        S::Sample: SizedSample + KnownSample,
        E: FnMut(StreamError) + Send + 'static,
    {
        self.build_output_stream(
            &signal.spec().into_cpal_config(),
            move |buf: &mut [S::Sample], _: &OutputCallbackInfo| {
                let n = match signal.read(buf) {
                    Ok(n) => n,
                    Err(PhonicError::NotReady)
                    | Err(PhonicError::Interrupted)
                    | Err(PhonicError::OutOfBounds) => 0,
                    Err(e) => panic!("error reading signal: {e}"),
                };

                buf[n..].fill(S::Sample::ORIGIN);
            },
            error_callback,
            timeout,
        )
    }
}

impl<T: DeviceTrait> DeviceExt for T {}

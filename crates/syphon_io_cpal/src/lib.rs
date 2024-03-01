use cpal::{
    traits::DeviceTrait, BuildStreamError, DefaultStreamConfigError, OutputCallbackInfo,
    SampleFormat, SampleRate, SizedSample, StreamConfig, StreamError, SupportedStreamConfig,
};
use std::time::Duration;
use syphon_io::{KnownSample, KnownSampleType};
use syphon_signal::{Signal, SignalReader, SignalSpec};

pub trait SignalSpecExtension {
    fn from_cpal_config(config: &StreamConfig) -> SignalSpec;
}

impl SignalSpecExtension for SignalSpec {
    fn from_cpal_config(config: &StreamConfig) -> SignalSpec {
        SignalSpec {
            frame_rate: config.sample_rate.0,
            channels: config.channels.into(),
            n_frames: None,
        }
    }
}

fn cpal_sample_from_syphon_sample(sample_type: KnownSampleType) -> SampleFormat {
    match sample_type {
        KnownSampleType::I8 => SampleFormat::I8,
        KnownSampleType::I16 => SampleFormat::I16,
        KnownSampleType::I32 => SampleFormat::I32,
        KnownSampleType::I64 => SampleFormat::I64,
        KnownSampleType::U8 => SampleFormat::U8,
        KnownSampleType::U16 => SampleFormat::U16,
        KnownSampleType::U32 => SampleFormat::U32,
        KnownSampleType::U64 => SampleFormat::U64,
        KnownSampleType::F32 => SampleFormat::F32,
        KnownSampleType::F64 => SampleFormat::F64,
    }
}

fn config_supports_signal<S>(config: &SupportedStreamConfig, signal: &S) -> bool
where
    S: Signal,
    S::Sample: KnownSample,
{
    let spec = signal.spec();

    config.sample_format() == cpal_sample_from_syphon_sample(S::Sample::TYPE)
        && config.channels() == spec.channels.count()
        && config.sample_rate() == SampleRate(spec.frame_rate)
}

pub trait DeviceExtension: DeviceTrait {
    fn build_output_stream_from_signal<S, E>(
        &self,
        mut signal: S,
        error_callback: E,
        timeout: Option<Duration>,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        S: SignalReader + std::marker::Send + 'static,
        S::Sample: SizedSample + KnownSample,
        E: FnMut(StreamError) + Send + 'static,
    {
        let config = self.default_output_config().map_err(|e| match e {
            DefaultStreamConfigError::DeviceNotAvailable => BuildStreamError::DeviceNotAvailable,
            DefaultStreamConfigError::StreamTypeNotSupported => {
                BuildStreamError::StreamConfigNotSupported
            }
            DefaultStreamConfigError::BackendSpecific { err } => {
                BuildStreamError::BackendSpecific { err }
            }
        })?;

        if !config_supports_signal(&config, &signal) {
            return Err(BuildStreamError::StreamConfigNotSupported);
        }

        self.build_output_stream(
            &config.into(),
            move |buf: &mut [S::Sample], _: &OutputCallbackInfo| {
                signal
                    .read_exact(buf)
                    .expect("error while reading from signal")
            },
            error_callback,
            timeout,
        )
    }
}

impl<T: DeviceTrait> DeviceExtension for T {}

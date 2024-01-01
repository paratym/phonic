use crate::{
    dsp::adapters::{ChannelsAdapter, DurationAdapter, FrameRateAdapter, SampleTypeAdapter},
    Channels, IntoSample, Sample, Signal, SignalReader, SignalSpec, SignalWriter,
};
use std::time::Duration;

pub trait SignalAdapter: Signal + Sized {
    fn adapt_sample_type<S: Sample>(self) -> SampleTypeAdapter<S, Self> {
        SampleTypeAdapter::new(self)
    }

    fn adapt_frame_rate(self, frame: u32) -> FrameRateAdapter<Self> {
        FrameRateAdapter::new(self, frame)
    }

    fn adapt_channels(self, channels: impl Into<Channels>) -> ChannelsAdapter<Self> {
        ChannelsAdapter::new(self, channels.into())
    }

    fn adapt_n_frames(self, n_frames: Option<u64>) -> DurationAdapter<Self> {
        DurationAdapter::new(self, n_frames)
    }

    fn adapt_duration(self, duration: Duration) -> DurationAdapter<Self> {
        DurationAdapter::from_duration(self, duration)
    }

    fn adapt_reader_spec<S, O>(self, spec: &SignalSpec) -> Box<dyn SignalReader<O>>
    where
        S: Sample + IntoSample<O> + 'static,
        O: Sample + 'static,
        Self: SignalReader<S> + Sized + 'static,
    {
        let src_spec = *self.spec();
        let mut adapter = Box::new(self.adapt_sample_type()) as Box<dyn SignalReader<O>>;

        if src_spec.frame_rate != spec.frame_rate {
            adapter = Box::new(adapter.adapt_frame_rate(spec.frame_rate));
        }

        if src_spec.channels != spec.channels {
            adapter = Box::new(adapter.adapt_channels(spec.channels));
        }

        if src_spec.n_frames != spec.n_frames {
            adapter = Box::new(adapter.adapt_n_frames(spec.n_frames));
        }

        adapter
    }

    fn adapt_writer_spec<S, O>(self, spec: &SignalSpec) -> Box<dyn SignalWriter<O>>
    where
        S: Sample + 'static,
        O: Sample + IntoSample<S> + 'static,
        Self: SignalWriter<S> + Sized + 'static,
    {
        let src_spec = *self.spec();
        let mut adapter = Box::new(self.adapt_sample_type()) as Box<dyn SignalWriter<O>>;

        if src_spec.frame_rate != spec.frame_rate {
            adapter = Box::new(adapter.adapt_frame_rate(spec.frame_rate));
        }

        if src_spec.channels != spec.channels {
            adapter = Box::new(adapter.adapt_channels(spec.channels));
        }

        if src_spec.n_frames != spec.n_frames {
            adapter = Box::new(adapter.adapt_n_frames(spec.n_frames));
        }

        adapter
    }
}

impl<T: Signal + Sized> SignalAdapter for T {}

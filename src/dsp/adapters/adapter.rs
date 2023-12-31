use std::time::Duration;
use crate::{Signal, Sample, Channels, FromSample, SignalSpec, dsp::adapters::{SampleTypeAdapter, FrameRateAdapter, ChannelsAdapter, DurationAdapter}, SignalReader, IntoSample, SignalWriter};

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

    // fn adapt_reader<O>(self, spec: &SignalSpec) -> Box<dyn SignalReader<O>>
    // where
    //     S: IntoSample<O> + 'static,
    //     O: Sample + 'static,
    //     Self: SignalReader<S> + Sized + 'static,
    // {
    //     let src_spec = *self.spec();
    //     let mut adapter = Box::new(self.adapt_sample_type()) as Box<dyn SignalReader<O>>;

    //     if src_spec.frame_rate != spec.frame_rate {
    //         adapter = Box::new(adapter.adapt_frame_rate(spec.frame_rate));
    //     }

    //     if src_spec.channels != spec.channels {
    //         adapter = Box::new(adapter.adapt_channels(spec.channels));
    //     }

    //     if src_spec.block_size != spec.block_size {
    //         adapter = Box::new(adapter.adapt_block_size(spec.block_size));
    //     }

    //     if src_spec.n_blocks != spec.n_blocks {
    //         adapter = Box::new(adapter.adapt_n_blocks(spec.n_blocks));
    //     }

    //     adapter 
    // }

    // fn adapt_writer<O>(self, spec: &SignalSpec) -> Box<dyn SignalWriter<O>>
    // where
    //     S: FromSample<O> + 'static,
    //     O: Sample + 'static,
    //     Self: SignalWriter<S> + Sized + 'static,
    // {
    //     let src_spec = *self.spec();
    //     let mut adapter = Box::new(self.adapt_sample_type()) as Box<dyn SignalWriter<O>>;

    //     if src_spec.frame_rate != spec.frame_rate {
    //         adapter = Box::new(adapter.adapt_frame_rate(spec.frame_rate));
    //     }

    //     if src_spec.channels != spec.channels {
    //         adapter = Box::new(adapter.adapt_channels(spec.channels));
    //     }

    //     if src_spec.block_size != spec.block_size {
    //         adapter = Box::new(adapter.adapt_block_size(spec.block_size));
    //     }

    //     if src_spec.n_blocks != spec.n_blocks {
    //         adapter = Box::new(adapter.adapt_n_blocks(spec.n_blocks));
    //     }

    //     adapter 
    // }
}

impl<T: Signal + Sized> SignalAdapter for T {}
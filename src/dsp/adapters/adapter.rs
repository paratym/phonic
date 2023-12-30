use std::time::Duration;
use crate::{Signal, Sample, Channels, FromSample, SignalSpec, dsp::adapters::{SampleTypeAdapter, FrameRateAdapter, ChannelsAdapter, BlockSizeAdapter, NBlocksAdapter}, SignalReader, IntoSample, SignalWriter};

pub trait SignalAdapter<S: Sample>: Signal<S> + Sized {
    fn adapt_sample_type<O: Sample>(self) -> SampleTypeAdapter<Self, S, O> {
        SampleTypeAdapter::new(self)
    }

    fn adapt_frame_rate(self, frame: u32) -> FrameRateAdapter<Self, S>
    where
        Self: Sized,
    {
        FrameRateAdapter::new(self, frame)
    }

    fn adapt_channels<C: Into<Channels>>(self, channels: C) -> ChannelsAdapter<Self, S>
    where
        Self: Sized,
    {
        ChannelsAdapter::new(self, channels.into())
    }

    fn adapt_block_size(self, block_size: usize) -> BlockSizeAdapter<Self, S>
    where
        Self: Sized,
    {
        BlockSizeAdapter::new(self, block_size)
    }

    fn adapt_n_blocks(self, n_blocks: Option<u64>) -> NBlocksAdapter<Self, S>
    where
        Self: Sized,
    {
        NBlocksAdapter::new(self, n_blocks)
    }

    fn adapt_seconds(self, seconds: f64) -> NBlocksAdapter<Self, S>
    where
        Self: Sized,
    {
        NBlocksAdapter::from_seconds(self, seconds)
    }

    fn adapt_duration(self, duration: Duration) -> NBlocksAdapter<Self, S>
    where
        Self: Sized,
    {
        NBlocksAdapter::from_duration(self, duration)
    }

    fn adapt_reader<O>(self, spec: &SignalSpec<O>) -> Box<dyn SignalReader<O>>
    where
        S: IntoSample<O> + 'static,
        O: Sample + 'static,
        Self: SignalReader<S> + Sized + 'static,
    {
        let src_spec = *self.spec();
        let mut adapter = Box::new(self.adapt_sample_type::<O>()) as Box<dyn SignalReader<O>>;

        if src_spec.frame_rate != spec.frame_rate {
            adapter = Box::new(adapter.adapt_frame_rate(spec.frame_rate));
        }

        if src_spec.channels != spec.channels {
            adapter = Box::new(adapter.adapt_channels(spec.channels));
        }

        if src_spec.block_size != spec.block_size {
            adapter = Box::new(adapter.adapt_block_size(spec.block_size));
        }

        if src_spec.n_blocks != spec.n_blocks {
            adapter = Box::new(adapter.adapt_n_blocks(spec.n_blocks));
        }

        adapter 
    }

    // fn adapt_writer<O>(self, spec: &SignalSpec<O>) -> Box<dyn SignalWriter<O>>
    // where
    //     S: FromSample<O> + 'static,
    //     O: Sample + 'static,
    //     Self: SignalWriter<S> + Sized + 'static,
    // {
    //     let src_spec = *self.spec();
    //     let mut adapter = Box::new(self.adapt_sample_type::<O>()) as Box<dyn SignalWriter<O>>;

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

impl<S: Sample, T: Signal<S> + Sized> SignalAdapter<S> for T {}
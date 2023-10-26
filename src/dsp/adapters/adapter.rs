use crate::{Signal, dsp::adapters::{BlockSizeAdapter, ChannelsAdapter, NBlocksAdapter, SampleRateAdapter, SampleTypeAdapter}, Sample, SignalReader, FromSample};

pub trait IntoAdapter<T: Signal, S: Sample> {
    fn adapt_block_size(self, block_size: usize) -> BlockSizeAdapter<T, S>;
    fn adapt_channels(self, n_channels: u8) -> ChannelsAdapter<T, S>;
    fn adapt_n_blocks(self, n_blocks: u64) -> NBlocksAdapter<T, S>;
    fn adapt_sample_rate(self, sample_rate: u32) -> SampleRateAdapter<T, S>;
    fn adapt_sample_type<O: Sample + FromSample<S>>(self) -> SampleTypeAdapter<T, S, O>;
}

impl<T: SignalReader<S>, S: Sample> IntoAdapter<T, S> for T {
    fn adapt_block_size(self, block_size: usize) -> BlockSizeAdapter<Self, S> {
        BlockSizeAdapter::from_signal(self, block_size)
    }

    fn adapt_channels(self, n_channels: u8) -> ChannelsAdapter<Self, S> {
        ChannelsAdapter::from_signal(self, n_channels)
    }

    fn adapt_n_blocks(self, n_blocks: u64) -> NBlocksAdapter<Self, S> {
        NBlocksAdapter::from_signal(self, n_blocks)
    }

    fn adapt_sample_rate(self, sample_rate: u32) -> SampleRateAdapter<Self, S> {
        SampleRateAdapter::from_signal(self, sample_rate)
    }

    fn adapt_sample_type<O: Sample + FromSample<S>>(self) -> SampleTypeAdapter<Self, S, O> {
        SampleTypeAdapter::from_signal(self)
    }
}

// impl<T: SignalWriter<S>, S: Sample> IntoAdapter<S> for T {}
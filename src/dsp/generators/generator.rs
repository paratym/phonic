use crate::{dsp::generators::SineGenerator, SignalSpecBuilder, SyphonError};

pub trait SignalGenerator: Into<SignalSpecBuilder> {
    fn generate_sine(self, frequency: f32) -> Result<SineGenerator, SyphonError> {
        Ok(SineGenerator::new(self.into().build()?, frequency))
    }
}

impl<T: Into<SignalSpecBuilder>> SignalGenerator for T {}

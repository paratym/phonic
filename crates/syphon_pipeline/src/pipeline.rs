use super::sample::{Sample, SampleFormat};

#[derive(Copy, Clone)]
pub struct SampleSpec {
    pub n_channels: u16,
    pub bits_per_sample: u16,
    pub sample_format: SampleFormat,
    pub sample_rate: u32,
}

pub struct Pipeline {
  pipes: (),
  outputs: (),
  sample_spec: SampleSpec,
  sample_buffer: ()
}

pub struct Connection {
    pipeline: Pipeline,
    source: Box<dyn Source>,
    outputs: HashMap<TypeId, Box<dyn Output>>,
}

impl Connection {
    pub fn new(pipeline: Pipeline) -> Self {
        Self {
            pipeline,
        }
    }
    
    pub fn set_source(&mut self, source: Box<dyn Source>) {
        self.source = source;
    }

    pub fn add_output<O: Output>(&mut self) {
        self.outputs.insert(TypeId::of::<O>(), Box::new(O::new()));
    }

    pub fn remove_output<O: Output>(&mut self) {
        self.outputs.remove(&TypeId::of::<O>());
    }
}
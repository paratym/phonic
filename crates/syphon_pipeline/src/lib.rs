mod pipeline;
pub use pipeline::*;

pub mod io;
pub mod sample;

pub mod prelude {
    pub use crate::{
        pipeline::{Connection, PipelineBuilder,
        io::Output,
        sample::{Sample, SampleFormat},
    };
}

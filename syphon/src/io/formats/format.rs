use crate::core::{SignalSpec, SignalSpecBuilder, SyphonError};
use std::io::{Read, Write};



pub trait FormatReader<K> {
    fn read_track_data(&mut self) -> Result<TrackDataBuilder<K>, SyphonError>;
    fn into_reader(self) -> Box<dyn Read>;
}

// pub trait FormatWriter: Write {
//     fn write_spec(&mut self, spec: SignalSpec) -> Result<(), SyphonError>;
// }

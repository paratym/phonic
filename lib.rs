#![doc = include_str!("README.md")]
#![doc(
    issue_tracker_base_url = "https://github.com/b3nb5n/phonic/issues",
    html_playground_url = "https://play.rust-lang.org/"
)]

pub use phonic_signal::*;

#[cfg(feature = "dsp")]
pub use phonic_dsp as dsp;

#[cfg(feature = "io")]
pub use phonic_io as io;

#[cfg(feature = "sync")]
pub use phonic_sync as sync;

#[cfg(feature = "cpal")]
pub use phonic_cpal as cpal;

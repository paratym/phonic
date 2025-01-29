use cpal::traits::{HostTrait, StreamTrait};
use phonic::{cpal::CpalSignal, dsp::utils::Osc, PhonicError, PhonicResult, SignalSpec};
use std::time::Duration;

fn main() -> PhonicResult<()> {
    let spec = SignalSpec::mono(48000);
    let signal = Osc::hz(440.0).amp(0.6).sin::<f32>(spec);

    let device = cpal::default_host()
        .default_output_device()
        .ok_or(PhonicError::not_found())?;

    let output = <CpalSignal>::new().build_output(&device, signal).unwrap();
    output.play().unwrap();

    std::thread::sleep(Duration::from_secs(1));

    Ok(())
}

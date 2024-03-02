use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{thread::sleep, time::Duration};
use syphon::{
    cpal::{DeviceExtension, SignalSpecExtension},
    signal::SignalSpec,
};

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no default output device");

    let config = device
        .default_output_config()
        .expect("no default ouput config")
        .config();

    // let spec = SignalSpec::from_cpal_config(&config);
    // let signal = SineGenerator::new(spec, 440.0);
    //
    // let output_stream = device
    //     .build_output_stream_from_signal(signal, |e| panic!("{}", e), None)
    //     .expect("error building output stream");
    //
    // output_stream.play().expect("error playing stream");
    // sleep(Duration::from_secs(1))
}

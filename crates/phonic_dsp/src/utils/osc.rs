use crate::ops::IntoSample;
use phonic_signal::{
    IndexedSignal, PhonicError, PhonicResult, Sample, Signal, SignalReader, SignalSeeker,
    SignalSpec,
};
use std::{f64::consts::PI, marker::PhantomData, mem::MaybeUninit};

pub struct Osc {
    pub frequency: f64,
    pub amplitude: f64,
    pub phase: f64,
}

impl Osc {
    pub fn hz(frequency: f64) -> Self {
        Self {
            frequency,
            amplitude: 1.0,
            phase: 0.0,
        }
    }

    pub fn amp(mut self, amplitude: f64) -> Self {
        self.amplitude = amplitude;
        self
    }

    pub fn phase(mut self, phase: f64) -> Self {
        self.phase = phase;
        self
    }
}

macro_rules! osc {
    ($($struct:ident : $fn:ident ($self:ident, $buf:ident) $sample:expr);+;) => {
        $(osc!($struct : ($self, $buf) $sample);)*

        impl Osc {
            $(pub fn $fn<S>(self, spec: SignalSpec) -> $struct<S> {
                let Self { frequency, amplitude, phase } = self;

                $struct {
                    spec,
                    _sample: PhantomData,

                    frequency,
                    amplitude,
                    phase,

                    pos: 0
                }
            })+
        }
    };
    ($struct:ident : ($self:ident, $buf:ident) $sample:expr) => {
        pub struct $struct<S> {
            pub spec: SignalSpec,
            pub _sample: PhantomData<S>,

            pub frequency: f64,
            pub amplitude: f64,
            pub phase: f64,

            pub pos: u64,
        }

        impl<S> $struct<S> {
            pub fn new(spec: SignalSpec, frequency: f64, amplitude: f64, phase: f64) -> Self {
                Self {
                    spec,
                    _sample: PhantomData,

                    frequency,
                    amplitude,
                    phase,

                    pos: 0
                }
            }

            pub fn hz(spec: SignalSpec, frequency: f64) -> Self {
                Self::new(spec, frequency, 1.0, 0.0)
            }

            #[inline]
            fn seconds(&self) -> f64 {
                self.pos as f64 / self.spec.sample_rate as f64
            }

            #[inline]
            fn sample(&$self) -> f64 {
                $sample
            }
        }

        impl<S: Sample> Signal for $struct<S> {
            type Sample = S;

            fn spec(&self) -> &SignalSpec {
                &self.spec
            }
        }

        impl<S: Sample> IndexedSignal for $struct<S> {
            fn pos(&self) -> u64 {
                self.pos
            }
        }

        impl<S: Sample> SignalReader for $struct<S>
        where
            f64: IntoSample<S>
        {
            fn read(&mut $self, $buf: &mut [MaybeUninit<S>]) -> PhonicResult<usize> {
                let frames = $buf.chunks_exact_mut($self.spec.n_channels);
                let n_frames = frames.len();

                for frame in frames {
                    let sample = $self.sample().into_sample();
                    frame.fill(MaybeUninit::new(sample));
                    $self.pos += 1;
                }

                Ok(n_frames * $self.spec.n_channels)
            }
        }

        impl<S: Sample> SignalSeeker for $struct<S> {
            fn seek(&mut self, offset: i64) -> PhonicResult<()> {
                self.pos = self.pos
                    .checked_add_signed(offset)
                    .ok_or(PhonicError::out_of_bounds())?;

                Ok(())
            }
        }
    };
}

osc! {
    Sin: sin(self, buf) {
        ((self.seconds() * self.frequency + self.phase) * PI * 2.0).sin() * self.amplitude
    };

    // Sqr: sqr(self, buf) {
    //     self.sin_sample().signum() * self.amplitude
    // };

    Tri: tri(self, buf) {
        (2.0 * self.amplitude / PI)
            * ((self.seconds() * self.frequency + self.phase) * PI * 2.0)
                .sin()
                .asin()
    };

    Saw: saw(self, buf) {
        (2.0 * self.amplitude / PI)
            * ((self.seconds() * self.frequency + self.phase) * PI)
                .tan()
                .atan()
    };

    // Ramp: ramp(self, buf) {
    //     self.saw_p_sample().neg()
    // };
}

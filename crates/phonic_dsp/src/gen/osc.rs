use crate::ops::IntoSample;
use phonic_core::PhonicError;
use phonic_signal::{IndexedSignal, Sample, Signal, SignalReader, SignalSeeker, SignalSpec};
use std::{
    f64::{self, consts::PI},
    marker::PhantomData,
    ops::Neg,
};

pub enum OscFn {
    Sin,
    Sqr,
    Tri,
    SawP,
    SawN,
}

pub struct Osc<S> {
    spec: SignalSpec,
    function: OscFn,
    _sample: PhantomData<S>,

    frequency: f64,
    amplitude: f64,
    phase: f64,

    pos: u64,
}

macro_rules! impl_construct {
    ($self:ident, $name:ident, $fn:ident) => {
        pub fn $name(spec: SignalSpec, frequency: f64, amplitude: f64, phase: f64) -> $self {
            $self::new(spec, OscFn::$fn, frequency, amplitude, phase)
        }
    };
}

impl<S> Osc<S> {
    pub fn new(
        spec: SignalSpec,
        function: OscFn,
        frequency: f64,
        amplitude: f64,
        phase: f64,
    ) -> Self {
        Self {
            spec,
            function,
            _sample: PhantomData,

            frequency,
            amplitude: amplitude.clamp(0.0, 1.0),
            phase: phase.clamp(0.0, 1.0),

            pos: 0,
        }
    }

    impl_construct!(Self, sin, Sin);
    impl_construct!(Self, tri, Tri);
    impl_construct!(Self, sqr, Sqr);
    impl_construct!(Self, saw_p, SawP);
    impl_construct!(Self, saw_n, SawN);

    fn ts(&self) -> f64 {
        self.pos as f64 / self.spec.sample_rate as f64
    }

    fn sin_sample(&self) -> f64 {
        ((self.ts() * self.frequency + self.phase) * PI * 2.0).sin() * self.amplitude
    }

    fn sqr_sample(&self) -> f64 {
        self.sin_sample().signum() * self.amplitude
    }

    fn tri_sample(&self) -> f64 {
        (2.0 * self.amplitude / PI)
            * ((self.ts() * self.frequency + self.phase) * PI * 2.0)
                .sin()
                .asin()
    }

    fn saw_p_sample(&self) -> f64 {
        (2.0 * self.amplitude / PI)
            * ((self.ts() * self.frequency + self.phase) * PI)
                .tan()
                .atan()
    }

    fn saw_n_sample(&self) -> f64 {
        self.saw_p_sample().neg()
    }
}

impl<S: Sample> Signal for Osc<S> {
    type Sample = S;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<S: Sample> IndexedSignal for Osc<S> {
    fn pos(&self) -> u64 {
        self.pos
    }
}

impl<S: Sample> SignalReader for Osc<S>
where
    f64: IntoSample<S>,
{
    fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        let n_channels = self.spec.channels.count() as usize;
        let frames = buf.chunks_exact_mut(n_channels);
        let n_frames = frames.len();

        let sample_fn = match self.function {
            OscFn::Sin => Self::sin_sample,
            OscFn::Sqr => Self::sqr_sample,
            OscFn::Tri => Self::tri_sample,
            OscFn::SawN => Self::saw_p_sample,
            OscFn::SawP => Self::saw_n_sample,
        };

        for frame in frames {
            frame.fill(sample_fn(self).into_sample());
            self.pos += 1;
        }

        Ok(n_frames * n_channels)
    }
}

impl<S: Sample> SignalSeeker for Osc<S> {
    fn seek(&mut self, offset: i64) -> Result<(), PhonicError> {
        self.pos = self
            .pos
            .checked_add_signed(offset)
            .ok_or(PhonicError::OutOfBounds)?;

        Ok(())
    }
}

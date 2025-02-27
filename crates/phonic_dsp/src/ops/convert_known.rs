use crate::ops::{Convert, FromSample, IntoSample};
use phonic_io::{
    dynamic::{DynSignal, TaggedSignal},
    match_tagged_signal,
};
use phonic_signal::{
    utils::{DefaultSizedBuf, SizedBuf},
    Sample,
};

pub trait FromKnownSample:
    Sample
    + FromSample<i8>
    + FromSample<i16>
    + FromSample<i32>
    + FromSample<i64>
    + FromSample<u8>
    + FromSample<u16>
    + FromSample<u32>
    + FromSample<u64>
    + FromSample<f32>
    + FromSample<f64>
{
}

pub trait IntoKnownSample:
    Sample
    + IntoSample<i8>
    + IntoSample<i16>
    + IntoSample<i32>
    + IntoSample<i64>
    + IntoSample<u8>
    + IntoSample<u16>
    + IntoSample<u32>
    + IntoSample<u64>
    + IntoSample<f32>
    + IntoSample<f64>
{
}

impl<S> FromKnownSample for S where
    S: Sample
        + FromSample<i8>
        + FromSample<i16>
        + FromSample<i32>
        + FromSample<i64>
        + FromSample<u8>
        + FromSample<u16>
        + FromSample<u32>
        + FromSample<u64>
        + FromSample<f32>
        + FromSample<f64>
{
}

impl<S> IntoKnownSample for S where
    S: Sample
        + IntoSample<i8>
        + IntoSample<i16>
        + IntoSample<i32>
        + IntoSample<i64>
        + IntoSample<u8>
        + IntoSample<u16>
        + IntoSample<u32>
        + IntoSample<u64>
        + IntoSample<f32>
        + IntoSample<f64>
{
}

pub trait TaggedSignalExt {
    fn convert<S>(self) -> Box<dyn DynSignal<Sample = S>>
    where
        S: FromKnownSample + IntoKnownSample;
}

impl TaggedSignalExt for TaggedSignal {
    fn convert<S>(self) -> Box<dyn DynSignal<Sample = S>>
    where
        S: FromKnownSample + IntoKnownSample,
    {
        match_tagged_signal!(self, signal => {
            let buf = DefaultSizedBuf::uninit();
            Box::new(<Convert<_, _>>::new(signal, buf))
        })
    }
}

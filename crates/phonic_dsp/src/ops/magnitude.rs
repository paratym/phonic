use phonic_signal::Sample;

pub trait SampleMagnitude: Sample {
    fn magnitude(&self) -> Self;
}

macro_rules! impl_magnitude {
    ($sample:ty, $self:ident, $func:expr) => {
        impl SampleMagnitude for $sample {
            fn magnitude(&$self) -> Self {
                $func
            }
        }
    };
}

impl_magnitude!(u8, self, self.abs_diff(Self::ORIGIN));
impl_magnitude!(u16, self, self.abs_diff(Self::ORIGIN));
impl_magnitude!(u32, self, self.abs_diff(Self::ORIGIN));
impl_magnitude!(u64, self, self.abs_diff(Self::ORIGIN));

impl_magnitude!(i8, self, self.saturating_abs());
impl_magnitude!(i16, self, self.saturating_abs());
impl_magnitude!(i32, self, self.saturating_abs());
impl_magnitude!(i64, self, self.saturating_abs());

impl_magnitude!(f32, self, self.abs());
impl_magnitude!(f64, self, self.abs());

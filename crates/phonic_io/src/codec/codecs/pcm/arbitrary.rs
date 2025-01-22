use phonic_signal::Sample;

pub unsafe trait ArbitrarySample: Sample {}

unsafe impl ArbitrarySample for u8 {}
unsafe impl ArbitrarySample for u16 {}
unsafe impl ArbitrarySample for u32 {}
unsafe impl ArbitrarySample for u64 {}

unsafe impl ArbitrarySample for i8 {}
unsafe impl ArbitrarySample for i16 {}
unsafe impl ArbitrarySample for i32 {}
unsafe impl ArbitrarySample for i64 {}

unsafe impl ArbitrarySample for f32 {}
unsafe impl ArbitrarySample for f64 {}

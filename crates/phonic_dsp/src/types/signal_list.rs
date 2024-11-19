use phonic_core::PhonicError;
use phonic_signal::{
    FiniteSignal, IndexedSignal, Sample, Signal, SignalReader, SignalSeeker, SignalSpec,
    SignalWriter,
};

pub trait SignalList {
    type Sample: Sample;

    fn spec(&self) -> Result<SignalSpec, PhonicError>;
    fn count(&self) -> usize;
}

pub trait IndexedSignalList: SignalList {
    fn pos(&self, i: usize) -> u64;

    fn min_pos(&self) -> u64 {
        let range = 0..self.count();
        range.map(|i| self.pos(i)).min().unwrap_or_default()
    }

    fn max_pos(&self) -> u64 {
        let range = 0..self.count();
        range.map(|i| self.pos(i)).max().unwrap_or_default()
    }
}

pub trait FiniteSignalList: SignalList {
    fn len(&self, i: usize) -> u64;

    fn min_len(&self) -> u64 {
        let range = 0..self.count();
        range.map(|i| self.len(i)).min().unwrap_or_default()
    }

    fn max_len(&self) -> u64 {
        let range = 0..self.count();
        range.map(|i| self.len(i)).max().unwrap_or_default()
    }
}

pub trait SignalReaderList: SignalList {
    fn read(&mut self, i: usize, buf: &mut [Self::Sample]) -> Result<usize, PhonicError>;
}

pub trait SignalWriterList: SignalList {
    fn write(&mut self, i: usize, buf: &[Self::Sample]) -> Result<usize, PhonicError>;
    fn flush(&mut self, i: usize) -> Result<(), PhonicError>;
}

pub trait SignalSeekerList: SignalList {
    fn seek(&mut self, i: usize, offset: i64) -> Result<(), PhonicError>;
}

impl<T: Signal, const N: usize> SignalList for [T; N] {
    type Sample = T::Sample;

    fn spec(&self) -> Result<SignalSpec, PhonicError> {
        let mut iter = self.iter().map(Signal::spec);
        let mut spec = *iter.next().ok_or(PhonicError::NotFound)?;
        for other in iter {
            spec.merge(other)?;
        }

        Ok(spec)
    }

    fn count(&self) -> usize {
        N
    }
}

impl<T: IndexedSignal, const N: usize> IndexedSignalList for [T; N] {
    fn pos(&self, i: usize) -> u64 {
        self[i].pos()
    }
}

impl<T: FiniteSignal, const N: usize> FiniteSignalList for [T; N] {
    fn len(&self, i: usize) -> u64 {
        self[i].len()
    }
}

impl<T: SignalReader, const N: usize> SignalReaderList for [T; N] {
    fn read(&mut self, i: usize, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
        self[i].read(buf)
    }
}

impl<T: SignalWriter, const N: usize> SignalWriterList for [T; N] {
    fn write(&mut self, i: usize, buf: &[Self::Sample]) -> Result<usize, PhonicError> {
        self[i].write(buf)
    }

    fn flush(&mut self, i: usize) -> Result<(), PhonicError> {
        self[i].flush()
    }
}

impl<T: SignalSeeker, const N: usize> SignalSeekerList for [T; N] {
    fn seek(&mut self, i: usize, offset: i64) -> Result<(), PhonicError> {
        self[i].seek(offset)
    }
}

macro_rules! impl_slice_list {
    ($signal:ident, $typ:ty) => {
        impl<$signal: Signal> SignalList for $typ {
            type Sample = $signal::Sample;

            fn spec(&self) -> Result<SignalSpec, PhonicError> {
                let mut iter = self.iter().map(Signal::spec);
                let mut spec = *iter.next().ok_or(PhonicError::NotFound)?;
                for other in iter {
                    spec.merge(other)?;
                }

                Ok(spec)
            }

            fn count(&self) -> usize {
                self.len()
            }
        }

        impl<$signal: IndexedSignal> IndexedSignalList for $typ {
            fn pos(&self, i: usize) -> u64 {
                self[i].pos()
            }
        }

        impl<$signal: FiniteSignal> FiniteSignalList for $typ {
            fn len(&self, i: usize) -> u64 {
                self[i].len()
            }
        }

        impl<$signal: SignalReader> SignalReaderList for $typ {
            fn read(&mut self, i: usize, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
                self[i].read(buf)
            }
        }

        impl<$signal: SignalWriter> SignalWriterList for $typ {
            fn write(&mut self, i: usize, buf: &[Self::Sample]) -> Result<usize, PhonicError> {
                self[i].write(buf)
            }

            fn flush(&mut self, i: usize) -> Result<(), PhonicError> {
                self[i].flush()
            }
        }

        impl<$signal: SignalSeeker> SignalSeekerList for $typ {
            fn seek(&mut self, i: usize, offset: i64) -> Result<(), PhonicError> {
                self[i].seek(offset)
            }
        }
    };
}

impl_slice_list!(T, Box<[T]>);
impl_slice_list!(T, Vec<T>);
impl_slice_list!(T, [T]);

macro_rules! count_repetition {
    () => {0usize};
    ($first:tt $($rest:tt)*) => (1usize + count_repetition!($($rest)*))
}

macro_rules! impl_tuple_list {
    ($first:ident = $first_i:tt, $($rest:ident = $rest_i:tt),+) => {
        impl<
            $first: Signal,
            $($rest: Signal<Sample = $first::Sample>),+
        > SignalList for ($first, $($rest),+) {
            type Sample = $first::Sample;

            fn spec(&self) -> Result<SignalSpec, PhonicError> {
                let mut spec = *self.$first_i.spec();
                $(spec.merge(self.$rest_i.spec())?);+;

                Ok(spec)
            }

            fn count(&self) -> usize {
                count_repetition!($first $($rest)+)
            }
        }

        impl<
            $first: IndexedSignal,
            $($rest: IndexedSignal<Sample = $first::Sample>),+
        > IndexedSignalList for ($first, $($rest),+) {
            fn pos(&self, i: usize) -> u64 {
                match i {
                    $first_i => self.$first_i.pos(),
                    $($rest_i => self.$rest_i.pos()),+,
                    _ => 0
                }
            }
        }

        impl<
            $first: FiniteSignal,
            $($rest: FiniteSignal<Sample = $first::Sample>),+
        > FiniteSignalList for ($first, $($rest),+) {
            fn len(&self, i: usize) -> u64 {
                match i {
                    $first_i => self.$first_i.len(),
                    $($rest_i => self.$rest_i.len()),+,
                    _ => 0
                }
            }
        }

        impl<
            $first: SignalReader,
            $($rest: SignalReader<Sample = $first::Sample>),+
        > SignalReaderList for ($first, $($rest),+) {
            fn read(&mut self, i: usize, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
                match i {
                    $first_i => self.$first_i.read(buf),
                    $($rest_i => self.$rest_i.read(buf)),+,
                    _ => Err(PhonicError::NotFound)
                }
            }
        }

        impl<
            $first: SignalWriter,
            $($rest: SignalWriter<Sample = $first::Sample>),+
        > SignalWriterList for ($first, $($rest),+) {
            fn write(&mut self, i: usize, buf: &[Self::Sample]) -> Result<usize, PhonicError> {
                match i {
                    $first_i => self.$first_i.write(buf),
                    $($rest_i => self.$rest_i.write(buf)),+,
                    _ => Err(PhonicError::NotFound)
                }
            }

            fn flush(&mut self, i: usize)  -> Result<(), PhonicError> {
                match i {
                    $first_i => self.$first_i.flush(),
                    $($rest_i => self.$rest_i.flush()),+,
                    _ => Err(PhonicError::NotFound)
                }
            }
        }

        impl<
            $first: SignalSeeker,
            $($rest: SignalSeeker<Sample = $first::Sample>),+
        > SignalSeekerList for ($first, $($rest),+) {
            fn seek(&mut self, i: usize, offset: i64) -> Result<(), PhonicError> {
                match i {
                    $first_i => self.$first_i.seek(offset),
                    $($rest_i => self.$rest_i.seek(offset)),+,
                    _ => Err(PhonicError::NotFound)
                }
            }
        }
    };
}

impl_tuple_list!(A = 0, B = 1);
impl_tuple_list!(A = 0, B = 1, C = 2);
impl_tuple_list!(A = 0, B = 1, C = 2, D = 3);
impl_tuple_list!(A = 0, B = 1, C = 2, D = 3, E = 4);
impl_tuple_list!(A = 0, B = 1, C = 2, D = 3, E = 4, F = 5);
impl_tuple_list!(A = 0, B = 1, C = 2, D = 3, E = 4, F = 5, G = 6);
impl_tuple_list!(A = 0, B = 1, C = 2, D = 3, E = 4, F = 5, G = 6, H = 7);
impl_tuple_list!(
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
    I = 8
);
impl_tuple_list!(
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
    I = 8,
    J = 9
);

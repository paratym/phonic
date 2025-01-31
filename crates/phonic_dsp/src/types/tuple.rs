use crate::types::{SignalList, SignalListMut};
use phonic_signal::{
    BlockingSignal, BufferedSignalReader, BufferedSignalWriter, FiniteSignal, IndexedSignal,
    PhonicResult, Signal, SignalReader, SignalSeeker, SignalSpec, SignalWriter,
};
use std::mem::MaybeUninit;

pub struct SignalTupleIndex<T> {
    list: T,
    idx: usize,
}

macro_rules! n_repetitions {
    () => { 0 };
    ($first:tt $($rest:tt)*) => { 1 + n_repetitions!($($rest)*) }
}

macro_rules! tuple_idx_ref {
    (
        & $([$mut:tt])? (
            $first:ident : $first_i:tt,
            $($rest:ident : $rest_i:tt),+
        )
    ) => {
        impl<
            $first: Signal,
            $($rest: Signal<Sample = $first::Sample>),+
        > SignalList for & $($mut)? ($first, $($rest),+) {
            type Sample = $first::Sample;

            type Signal<'a> = SignalTupleIndex<&'a ($first, $($rest),+)>
            where
                Self: 'a;

            fn len(&self) -> usize {
                const { n_repetitions!($first $($rest)*) }
            }

            fn signal(&self, idx: usize) -> Self::Signal<'_> {
                assert!(idx < self.len(), "tuple index out of bounds");

                SignalTupleIndex {
                    list: *self,
                    idx
                }
            }
        }

        impl<
            $first: Signal,
            $($rest: Signal<Sample = $first::Sample>),+
        > Signal for SignalTupleIndex<& $($mut)? ($first, $($rest),+)> {
            type Sample = $first::Sample;

            fn spec(&self) -> &SignalSpec {
                match self.idx {
                    $first_i => &self.list.$first_i.spec(),
                    $($rest_i => &self.list.$rest_i.spec()),+,
                    _ => unreachable!()
                }
            }
        }

        impl<
            $first: BlockingSignal,
            $($rest: BlockingSignal<Sample = $first::Sample>),+
        > BlockingSignal for SignalTupleIndex<& $($mut)? ($first, $($rest),+)> {
            fn block(&self) {
                match self.idx {
                    $first_i => self.list.$first_i.block(),
                    $($rest_i => self.list.$rest_i.block()),+,
                    _ => unreachable!()
                }
            }
        }

        impl<
            $first: IndexedSignal,
            $($rest: IndexedSignal<Sample = $first::Sample>),+
        > IndexedSignal for SignalTupleIndex<& $($mut)? ($first, $($rest),+)> {
            fn pos(&self) -> u64 {
                match self.idx {
                    $first_i => self.list.$first_i.pos(),
                    $($rest_i => self.list.$rest_i.pos()),+,
                    _ => unreachable!()
                }
            }
        }

        impl<
            $first: FiniteSignal,
            $($rest: FiniteSignal<Sample = $first::Sample>),+
        > FiniteSignal for SignalTupleIndex<& $($mut)? ($first, $($rest),+)> {
            fn len(&self) -> u64 {
                match self.idx {
                    $first_i => self.list.$first_i.len(),
                    $($rest_i => self.list.$rest_i.len()),+,
                    _ => unreachable!()
                }
            }
        }
    };
}

macro_rules! tuple_idx_mut {
    (
        & [mut] (
            $first:ident : $first_i:tt,
            $($rest:ident : $rest_i:tt),+
        )
    ) => {
        impl<
            $first: Signal,
            $($rest: Signal<Sample = $first::Sample>),+
        > SignalListMut for &mut ($first, $($rest),+) {
            type SignalMut<'a> = SignalTupleIndex<&'a mut ($first, $($rest),+)>
            where
                Self: 'a;

            fn signal_mut(&mut self, idx: usize) -> Self::SignalMut<'_> {
                assert!(idx < self.len(), "tuple index out of bounds");

                SignalTupleIndex {
                    list: self,
                    idx
                }
            }
        }

        impl<
            $first: SignalReader,
            $($rest: SignalReader<Sample = $first::Sample>),+
        > SignalReader for SignalTupleIndex<&mut ($first, $($rest),+)> {
            fn read(&mut self, buf: &mut [MaybeUninit<Self::Sample>]) -> PhonicResult<usize> {
                match self.idx {
                    $first_i => self.list.$first_i.read(buf),
                    $($rest_i => self.list.$rest_i.read(buf)),+,
                    _ => unreachable!()
                }
            }
        }

        impl<
            $first: BufferedSignalReader,
            $($rest: BufferedSignalReader<Sample = $first::Sample>),+
        > BufferedSignalReader for SignalTupleIndex<&mut ($first, $($rest),+)> {
            fn fill(&mut self) -> PhonicResult<&[Self::Sample]> {
                match self.idx {
                    $first_i => self.list.$first_i.fill(),
                    $($rest_i => self.list.$rest_i.fill()),+,
                    _ => unreachable!()
                }
            }

            fn buffer(&self) -> Option<&[Self::Sample]> {
                match self.idx {
                    $first_i => self.list.$first_i.buffer(),
                    $($rest_i => self.list.$rest_i.buffer()),+,
                    _ => unreachable!()
                }
            }

            fn consume(&mut self, n_samples: usize) {
                match self.idx {
                    $first_i => self.list.$first_i.consume(n_samples),
                    $($rest_i => self.list.$rest_i.consume(n_samples)),+,
                    _ => unreachable!()
                }
            }
        }

        impl<
            $first: SignalWriter,
            $($rest: SignalWriter<Sample = $first::Sample>),+
        > SignalWriter for SignalTupleIndex<&mut ($first, $($rest),+)> {
            fn write(&mut self, buf: &[Self::Sample]) -> PhonicResult<usize> {
                match self.idx {
                    $first_i => self.list.$first_i.write(buf),
                    $($rest_i => self.list.$rest_i.write(buf)),+,
                    _ => unreachable!()
                }
            }

            fn flush(&mut self) -> PhonicResult<()> {
                match self.idx {
                    $first_i => self.list.$first_i.flush(),
                    $($rest_i => self.list.$rest_i.flush()),+,
                    _ => unreachable!()
                }
            }
        }

        impl<
            $first: BufferedSignalWriter,
            $($rest: BufferedSignalWriter<Sample = $first::Sample>),+
        > BufferedSignalWriter for SignalTupleIndex<&mut ($first, $($rest),+)> {
            fn buffer_mut(&mut self) -> Option<&mut [MaybeUninit<Self::Sample>]> {
                match self.idx {
                    $first_i => self.list.$first_i.buffer_mut(),
                    $($rest_i => self.list.$rest_i.buffer_mut()),+,
                    _ => unreachable!()
                }
            }

            fn commit(&mut self, n_samples: usize) {
                match self.idx {
                    $first_i => self.list.$first_i.commit(n_samples),
                    $($rest_i => self.list.$rest_i.commit(n_samples)),+,
                    _ => unreachable!()
                }
            }
        }

        impl<
            $first: SignalSeeker,
            $($rest: SignalSeeker<Sample = $first::Sample>),+
        > SignalSeeker for SignalTupleIndex<&mut ($first, $($rest),+)> {
            fn seek(&mut self, offset: i64) -> PhonicResult<()> {
                match self.idx {
                    $first_i => self.list.$first_i.seek(offset),
                    $($rest_i => self.list.$rest_i.seek(offset)),+,
                    _ => unreachable!()
                }
            }
        }
    };
}

macro_rules! tuple_idx {
    (
        $first:ident : $first_i:tt,
        $($rest:ident : $rest_i:tt),+
    ) => {
        impl<
            $first: Signal,
            $($rest: Signal<Sample = $first::Sample>),+
        > SignalList for ($first, $($rest),+) {
            type Sample = $first::Sample;

            type Signal<'a> = SignalTupleIndex<&'a Self>
            where
                Self: 'a;

            fn len(&self) -> usize {
                const { n_repetitions!($first $($rest)*) }
            }

            fn signal(&self, idx: usize) -> Self::Signal<'_> {
                assert!(idx < self.len(), "tuple index out of bounds");

                SignalTupleIndex {
                    list: self,
                    idx
                }
            }
        }

        impl<
            $first: Signal,
            $($rest: Signal<Sample = $first::Sample>),+
        > SignalListMut for ($first, $($rest),+) {
            type SignalMut<'a> = SignalTupleIndex<&'a mut Self>
            where
                Self: 'a;

            fn signal_mut(&mut self, idx: usize) -> Self::SignalMut<'_> {
                assert!(idx < self.len(), "tuple index out of bounds");

                SignalTupleIndex {
                    list: self,
                    idx
                }
            }
        }

        tuple_idx_ref! {
            &(
                $first : $first_i,
                $($rest : $rest_i),+
            )
        }

        tuple_idx_ref! {
            &[mut] (
                $first : $first_i,
                $($rest : $rest_i),+
            )
        }

        tuple_idx_mut! {
            &[mut] (
                $first : $first_i,
                $($rest : $rest_i),+
            )
        }
    };
}

tuple_idx!(A: 0, B: 1);
tuple_idx!(A: 0, B: 1, C: 2);
tuple_idx!(A: 0, B: 1, C: 2, D: 3);
tuple_idx!(A: 0, B: 1, C: 2, D: 3, E: 4);
tuple_idx!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
tuple_idx!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
tuple_idx!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);
tuple_idx!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8);
tuple_idx!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9);
tuple_idx!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10);
tuple_idx!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11);
tuple_idx!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12);

use phonic_signal::{PhonicError, PhonicResult, Sample, Signal, SignalSpec};
use std::{rc::Rc, sync::Arc};

pub trait SignalList {
    type Sample: Sample;

    type Signal<'a>: 'a + Signal<Sample = Self::Sample>
    where
        Self: 'a;

    fn len(&self) -> usize;
    fn signal(&self, idx: usize) -> Self::Signal<'_>;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn merged_spec(&self) -> PhonicResult<SignalSpec> {
        if self.is_empty() {
            return Err(PhonicError::missing_data());
        }

        let spec = *self.signal(0).spec();
        for idx in 1..self.len() {
            if self.signal(idx).spec() != &spec {
                return Err(PhonicError::param_mismatch());
            }
        }

        Ok(spec)
    }
}

pub trait SignalListMut: SignalList {
    type SignalMut<'a>: 'a + Signal<Sample = Self::Sample>
    where
        Self: 'a;

    fn signal_mut(&mut self, idx: usize) -> Self::SignalMut<'_>;
}

impl<T: Signal> SignalList for &[T] {
    type Sample = T::Sample;

    type Signal<'a>
        = &'a T
    where
        Self: 'a;

    fn len(&self) -> usize {
        (**self).len()
    }

    fn signal(&self, idx: usize) -> Self::Signal<'_> {
        &self[idx]
    }
}

impl<T: Signal> SignalList for Rc<[T]> {
    type Sample = T::Sample;

    type Signal<'a>
        = &'a T
    where
        Self: 'a;

    fn len(&self) -> usize {
        (**self).len()
    }

    fn signal(&self, idx: usize) -> Self::Signal<'_> {
        &self[idx]
    }
}

impl<T: Signal> SignalList for Arc<[T]> {
    type Sample = T::Sample;

    type Signal<'a>
        = &'a T
    where
        Self: 'a;

    fn len(&self) -> usize {
        (**self).len()
    }

    fn signal(&self, idx: usize) -> Self::Signal<'_> {
        &self[idx]
    }
}

impl<T: Signal> SignalList for &mut [T] {
    type Sample = T::Sample;

    type Signal<'a>
        = &'a T
    where
        Self: 'a;

    fn len(&self) -> usize {
        (**self).len()
    }

    fn signal(&self, idx: usize) -> Self::Signal<'_> {
        &self[idx]
    }
}

impl<T: Signal> SignalListMut for &mut [T] {
    type SignalMut<'a>
        = &'a mut T
    where
        Self: 'a;

    fn signal_mut(&mut self, idx: usize) -> Self::SignalMut<'_> {
        &mut self[idx]
    }
}

impl<T: Signal, const N: usize> SignalList for [T; N] {
    type Sample = T::Sample;

    type Signal<'a>
        = &'a T
    where
        Self: 'a;

    fn len(&self) -> usize {
        N
    }

    fn signal(&self, idx: usize) -> Self::Signal<'_> {
        &self[idx]
    }
}

impl<T: Signal, const N: usize> SignalListMut for [T; N] {
    type SignalMut<'a>
        = &'a mut T
    where
        Self: 'a;

    fn signal_mut(&mut self, idx: usize) -> Self::SignalMut<'_> {
        &mut self[idx]
    }
}

impl<T: Signal> SignalList for Vec<T> {
    type Sample = T::Sample;

    type Signal<'a>
        = &'a T
    where
        Self: 'a;

    fn len(&self) -> usize {
        (**self).len()
    }

    fn signal(&self, idx: usize) -> Self::Signal<'_> {
        &self[idx]
    }
}

impl<T: Signal> SignalListMut for Vec<T> {
    type SignalMut<'a>
        = &'a mut T
    where
        Self: 'a;

    fn signal_mut(&mut self, idx: usize) -> Self::SignalMut<'_> {
        &mut self[idx]
    }
}

impl<T: Signal> SignalList for Box<[T]> {
    type Sample = T::Sample;

    type Signal<'a>
        = &'a T
    where
        Self: 'a;

    fn len(&self) -> usize {
        (**self).len()
    }

    fn signal(&self, idx: usize) -> Self::Signal<'_> {
        &self[idx]
    }
}

impl<T: Signal> SignalListMut for Box<[T]> {
    type SignalMut<'a>
        = &'a mut T
    where
        Self: 'a;

    fn signal_mut(&mut self, idx: usize) -> Self::SignalMut<'_> {
        &mut self[idx]
    }
}

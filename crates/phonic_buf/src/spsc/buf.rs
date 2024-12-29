use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};

pub struct SpscBuf<T, B> {
    _buf: B,
    _element: PhantomData<T>,

    buf_ptr: *mut MaybeUninit<T>,
    buf_cap: usize,

    /// the index of the next slot to write to.
    /// in the rage 0 <= w_idx < buf_cap
    w_idx: AtomicUsize,

    /// the index of the next slot to read from.
    /// in the range 0 <= r_idx < buf_cap
    r_idx: AtomicUsize,

    /// only used to differentiate between an empty and a full buffer when r_idx == w_idx.
    /// if both indecies are equal and the last operation was a write the buffer must be full.
    trailing_write: AtomicBool,
}

pub struct Producer<T, B> {
    inner: Arc<SpscBuf<T, B>>,
}

pub struct Consumer<T, B> {
    inner: Arc<SpscBuf<T, B>>,
}

impl<T, B> SpscBuf<T, B> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(mut buf: B) -> (Producer<T, B>, Consumer<T, B>)
    where
        B: AsMut<[MaybeUninit<T>]>,
    {
        let slice = buf.as_mut();
        let buf_ptr = slice.as_mut_ptr();
        let buf_cap = slice.len();

        let inner = Self {
            buf_ptr,
            buf_cap,

            r_idx: AtomicUsize::default(),
            w_idx: AtomicUsize::default(),
            trailing_write: AtomicBool::default(),

            _buf: buf,
            _element: PhantomData,
        };

        let inner_ref = Arc::new(inner);
        (Producer::from(inner_ref.clone()), Consumer::from(inner_ref))
    }

    unsafe fn drop_elements(&self, start: usize, end: usize) {
        if start == end && !self.trailing_write.load(Ordering::Acquire) {
            return;
        }

        let mut ptr: *mut T = self.buf_ptr.add(start).cast();
        let end_ptr: *mut T = self.buf_ptr.add(end).cast();
        let wrap_ptr: *mut T = self.buf_ptr.add(self.buf_cap).cast();

        loop {
            ptr.drop_in_place();
            ptr = ptr.add(1);

            if ptr == wrap_ptr {
                ptr = self.buf_ptr.cast();
            }

            if ptr == end_ptr {
                break;
            }
        }
    }
}

impl<T, B> From<Arc<SpscBuf<T, B>>> for Producer<T, B> {
    fn from(inner: Arc<SpscBuf<T, B>>) -> Self {
        Self { inner }
    }
}

impl<T, B> From<Arc<SpscBuf<T, B>>> for Consumer<T, B> {
    fn from(inner: Arc<SpscBuf<T, B>>) -> Self {
        Self { inner }
    }
}

impl<T, B> Producer<T, B> {
    pub fn slots(&mut self) -> (&mut [MaybeUninit<T>], &mut [MaybeUninit<T>]) {
        let w_idx = self.inner.w_idx.load(Ordering::Relaxed);
        let r_idx = self.inner.r_idx.load(Ordering::Acquire);

        if w_idx < r_idx {
            let slots = unsafe {
                let slot_ptr = self.inner.buf_ptr.add(w_idx);
                let n_slots = r_idx - w_idx;

                std::slice::from_raw_parts_mut(slot_ptr, n_slots)
            };

            return (slots, &mut []);
        }

        if w_idx == r_idx && self.inner.trailing_write.load(Ordering::SeqCst) {
            return (&mut [], &mut []);
        }

        let trailing_slots = unsafe {
            let slot_ptr = self.inner.buf_ptr.add(w_idx);
            std::slice::from_raw_parts_mut(slot_ptr, self.inner.buf_cap - w_idx)
        };

        let leading_slots = unsafe {
            let slot_ptr = self.inner.buf_ptr;
            std::slice::from_raw_parts_mut(slot_ptr, r_idx)
        };

        (trailing_slots, leading_slots)
    }

    pub fn commit(&mut self, n: usize) {
        let w_idx = self.inner.w_idx.load(Ordering::Relaxed);
        let end_idx = w_idx + n % self.inner.buf_cap;

        self.inner.w_idx.store(end_idx, Ordering::Relaxed);
        self.inner.trailing_write.store(true, Ordering::SeqCst);
    }

    pub fn is_abandoned(&self) -> bool {
        Arc::strong_count(&self.inner) < 2
    }
}

impl<T, B> Consumer<T, B> {
    pub fn elements(&self) -> (&[T], &[T]) {
        let w_idx = self.inner.w_idx.load(Ordering::Acquire);
        let r_idx = self.inner.r_idx.load(Ordering::Relaxed);

        if r_idx < w_idx {
            let elements = unsafe {
                let slot_ptr = self.inner.buf_ptr.add(r_idx);
                let element_ptr = slot_ptr.cast();
                let n_elements = w_idx - r_idx;

                std::slice::from_raw_parts(element_ptr, n_elements)
            };

            return (elements, &[]);
        }

        if r_idx == w_idx && !self.inner.trailing_write.load(Ordering::SeqCst) {
            return (&[], &[]);
        }

        let trailing_elements = unsafe {
            let slot_ptr = self.inner.buf_ptr.add(r_idx);
            let element_ptr = slot_ptr.cast();
            let n_elements = self.inner.buf_cap - r_idx;

            std::slice::from_raw_parts(element_ptr, n_elements)
        };

        let leading_elements = unsafe {
            let slot_ptr = self.inner.buf_ptr;
            let element_ptr = slot_ptr.cast();
            let n_elements = w_idx;

            std::slice::from_raw_parts(element_ptr, n_elements)
        };

        (trailing_elements, leading_elements)
    }

    pub fn commit(&mut self, n: usize) {
        let r_idx = self.inner.r_idx.load(Ordering::Relaxed);
        let end_idx = r_idx + n % self.inner.buf_cap;

        unsafe { self.inner.drop_elements(r_idx, end_idx) };

        self.inner.r_idx.store(end_idx, Ordering::Relaxed);
        self.inner.trailing_write.store(false, Ordering::SeqCst);
        // TODO: can trailing write interactions be relaxed to Acquire/Release ordering?
    }

    pub fn is_abandoned(&self) -> bool {
        Arc::strong_count(&self.inner) < 2
    }
}

unsafe impl<T: Send, B: Send> Send for Consumer<T, B> {}

impl<T, B> Drop for SpscBuf<T, B> {
    fn drop(&mut self) {
        let w_idx = self.w_idx.load(Ordering::Relaxed);
        let r_idx = self.r_idx.load(Ordering::Relaxed);

        unsafe { self.drop_elements(r_idx, w_idx) }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::clone_on_copy)]

    use super::*;

    const BUF_LEN: usize = 16;
    const BUF: [MaybeUninit<usize>; BUF_LEN] = [MaybeUninit::uninit(); BUF_LEN];

    #[test]
    fn new_buf_has_all_slots_and_no_elements() {
        let (mut producer, consumer) = SpscBuf::new(BUF.clone());

        assert_eq!(producer.slots().0.len(), BUF_LEN);
        assert_eq!(producer.slots().1.len(), 0);

        assert_eq!(consumer.elements().0.len(), 0);
        assert_eq!(consumer.elements().1.len(), 0);
    }

    #[test]
    fn committed_slots_are_moved_to_consumer() {
        let (mut producer, consumer) = SpscBuf::new(BUF.clone());

        let (slots, _) = producer.slots();
        slots.iter_mut().enumerate().for_each(|(i, slot)| {
            slot.write(i);
        });

        producer.commit(BUF_LEN);

        let (leading_slots, trailing_slots) = producer.slots();
        assert_eq!(leading_slots.len(), 0);
        assert_eq!(trailing_slots.len(), 0);

        let (leading_elements, trailing_elements) = consumer.elements();
        assert_eq!(leading_elements.len(), BUF_LEN);
        assert_eq!(trailing_elements.len(), 0);

        assert!(leading_elements
            .iter()
            .enumerate()
            .all(|(i, element)| *element == i))
    }

    #[test]
    fn producer_is_abandoned_when_consumer_is_dropped() {
        let (producer, consumer) = SpscBuf::new(BUF.clone());

        assert!(!producer.is_abandoned());
        assert!(!consumer.is_abandoned());

        drop(consumer);
        assert!(producer.is_abandoned());
    }

    #[test]
    fn consumer_is_abandoned_when_producer_is_dropped() {
        let (producer, consumer) = SpscBuf::new(BUF.clone());

        assert!(!producer.is_abandoned());
        assert!(!consumer.is_abandoned());

        drop(producer);
        assert!(consumer.is_abandoned());
    }

    // #[test]
    // fn elements_are_dropped_after_they_are_consumed() {
    //     let (producer, consumer) = SpscBuf::new(BUF.clone());
    // }
}

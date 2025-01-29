use std::{
    mem::{needs_drop, MaybeUninit},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};

pub struct SpscBuf<T, B> {
    _buf: B,
    ptr: *mut MaybeUninit<T>,
    cap: usize,

    /// the index of the next slot to write to.
    /// in the rage 0 <= w_idx < buf_cap
    w_idx: AtomicUsize,

    /// the index of the next slot to read from.
    /// in the range 0 <= r_idx < buf_cap
    r_idx: AtomicUsize,

    /// only used to differentiate between an empty and a full buffer when r_idx == w_idx.
    /// if both indices are equal and the last operation was a write the buffer must be full.
    trailing_write: AtomicBool,
}

pub struct Producer<T, B> {
    buf: Arc<SpscBuf<T, B>>,
}

pub struct Consumer<T, B> {
    buf: Arc<SpscBuf<T, B>>,
}

type SpscPair<T, B> = (Producer<T, B>, Consumer<T, B>);

impl<T, B> SpscBuf<T, B> {
    pub unsafe fn from_raw_parts(buf: B, ptr: *mut MaybeUninit<T>, cap: usize) -> SpscPair<T, B> {
        let inner = Self {
            _buf: buf,
            ptr,
            cap,

            w_idx: Default::default(),
            r_idx: Default::default(),
            trailing_write: Default::default(),
        };

        let inner_ref = Arc::new(inner);
        (Producer::new(inner_ref.clone()), Consumer::new(inner_ref))
    }

    #[allow(clippy::new_ret_no_self)]
    pub fn new(mut buf: B) -> SpscPair<T, B>
    where
        B: AsMut<[T]>,
    {
        let slice = buf.as_mut();
        let ptr = slice.as_mut_ptr().cast();
        let cap = slice.len();

        unsafe { Self::from_raw_parts(buf, ptr, cap) }
    }

    pub fn new_uninit(mut buf: B) -> SpscPair<T, B>
    where
        B: AsMut<[MaybeUninit<T>]>,
    {
        let slice = buf.as_mut();
        let ptr = slice.as_mut_ptr();
        let cap = slice.len();

        unsafe { Self::from_raw_parts(buf, ptr, cap) }
    }

    unsafe fn drop_elements(&self, start: usize, end: usize) {
        if !needs_drop::<T>() {
            return;
        }

        let mut ptr: *mut T = self.ptr.add(start).cast();
        let end_ptr: *mut T = self.ptr.add(end).cast();
        let wrap_ptr: *mut T = self.ptr.add(self.cap).cast();

        loop {
            ptr.drop_in_place();
            ptr = ptr.add(1);

            if ptr == wrap_ptr {
                ptr = self.ptr.cast();
            }

            if ptr == end_ptr {
                break;
            }
        }
    }
}

impl<T, B> Producer<T, B> {
    fn new(buf: Arc<SpscBuf<T, B>>) -> Self {
        Self { buf }
    }

    pub fn slots(&mut self) -> (&mut [MaybeUninit<T>], &mut [MaybeUninit<T>]) {
        let w_idx = self.buf.w_idx.load(Ordering::Relaxed);
        let r_idx = self.buf.r_idx.load(Ordering::Acquire);

        if w_idx < r_idx {
            let slots = unsafe {
                let slot_ptr = self.buf.ptr.add(w_idx);
                let n_slots = r_idx - w_idx;

                std::slice::from_raw_parts_mut(slot_ptr, n_slots)
            };

            return (slots, &mut []);
        }

        if w_idx == r_idx && self.buf.trailing_write.load(Ordering::SeqCst) {
            return (&mut [], &mut []);
        }

        let trailing_slots = unsafe {
            let slot_ptr = self.buf.ptr.add(w_idx);
            std::slice::from_raw_parts_mut(slot_ptr, self.buf.cap - w_idx)
        };

        let leading_slots = unsafe {
            let slot_ptr = self.buf.ptr;
            std::slice::from_raw_parts_mut(slot_ptr, r_idx)
        };

        (trailing_slots, leading_slots)
    }

    pub fn commit(&mut self, n: usize) {
        let w_idx = self.buf.w_idx.load(Ordering::Relaxed);
        let end_idx = w_idx + n % self.buf.cap;

        self.buf.w_idx.store(end_idx, Ordering::Relaxed);
        self.buf.trailing_write.store(true, Ordering::SeqCst);
    }

    pub fn is_empty(&self) -> bool {
        let w_idx = self.buf.w_idx.load(Ordering::Relaxed);
        let r_idx = self.buf.r_idx.load(Ordering::Acquire);

        w_idx == r_idx && !self.buf.trailing_write.load(Ordering::SeqCst)
    }

    pub fn is_full(&self) -> bool {
        let w_idx = self.buf.w_idx.load(Ordering::Relaxed);
        let r_idx = self.buf.r_idx.load(Ordering::Acquire);

        w_idx == r_idx && self.buf.trailing_write.load(Ordering::SeqCst)
    }

    pub fn is_abandoned(&self) -> bool {
        Arc::strong_count(&self.buf) < 2
    }
}

impl<T, B> Consumer<T, B> {
    fn new(buf: Arc<SpscBuf<T, B>>) -> Self {
        Self { buf }
    }

    pub fn elements(&self) -> (&[T], &[T]) {
        let w_idx = self.buf.w_idx.load(Ordering::Acquire);
        let r_idx = self.buf.r_idx.load(Ordering::Relaxed);

        if r_idx < w_idx {
            let elements = unsafe {
                let slot_ptr = self.buf.ptr.add(r_idx);
                let element_ptr = slot_ptr.cast();
                let n_elements = w_idx - r_idx;

                std::slice::from_raw_parts(element_ptr, n_elements)
            };

            return (elements, &[]);
        }

        if r_idx == w_idx && !self.buf.trailing_write.load(Ordering::SeqCst) {
            return (&[], &[]);
        }

        let trailing_elements = unsafe {
            let slot_ptr = self.buf.ptr.add(r_idx);
            let element_ptr = slot_ptr.cast();
            let n_elements = self.buf.cap - r_idx;

            std::slice::from_raw_parts(element_ptr, n_elements)
        };

        let leading_elements = unsafe {
            let slot_ptr = self.buf.ptr;
            let element_ptr = slot_ptr.cast();
            let n_elements = w_idx;

            std::slice::from_raw_parts(element_ptr, n_elements)
        };

        (trailing_elements, leading_elements)
    }

    pub fn consume(&mut self, n: usize) {
        if n == 0 {
            // this check is necessary because drop_elements assumes the buffer is always full when
            // start == end
            return;
        }

        let r_idx = self.buf.r_idx.load(Ordering::Relaxed);
        let end_idx = r_idx + n % self.buf.cap;

        unsafe { self.buf.drop_elements(r_idx, end_idx) };

        self.buf.r_idx.store(end_idx, Ordering::Relaxed);
        self.buf.trailing_write.store(false, Ordering::SeqCst);
        // TODO: can trailing write interactions be relaxed to Acquire/Release ordering?
    }

    pub fn is_empty(&self) -> bool {
        let w_idx = self.buf.w_idx.load(Ordering::Acquire);
        let r_idx = self.buf.r_idx.load(Ordering::Relaxed);

        r_idx == w_idx && !self.buf.trailing_write.load(Ordering::SeqCst)
    }

    pub fn is_full(&self) -> bool {
        let w_idx = self.buf.w_idx.load(Ordering::Acquire);
        let r_idx = self.buf.r_idx.load(Ordering::Relaxed);

        r_idx == w_idx && self.buf.trailing_write.load(Ordering::SeqCst)
    }

    pub fn is_abandoned(&self) -> bool {
        Arc::strong_count(&self.buf) < 2
    }
}

unsafe impl<T: Send, B: Send> Send for Consumer<T, B> {}

impl<T, B> Drop for SpscBuf<T, B> {
    fn drop(&mut self) {
        let w_idx = self.w_idx.load(Ordering::Relaxed);
        let r_idx = self.r_idx.load(Ordering::Relaxed);

        if r_idx == w_idx && !self.trailing_write.load(Ordering::Relaxed) {
            return;
        }

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
        let (mut producer, consumer) = SpscBuf::new_uninit(BUF.clone());

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

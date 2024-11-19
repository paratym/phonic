use crate::types::SpmcRingBuf;
use phonic_core::PhonicError;
use phonic_signal::{DefaultBuf, FiniteSignal, IndexedSignal, Signal, SignalReader, SignalSpec};
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

struct SplitInner<T, B> {
    inner: T,
    buf: SpmcRingBuf<B>,
}

pub struct Split<T: Signal, B = DefaultBuf<<T as Signal>::Sample>> {
    id: usize,
    spec: SignalSpec,
    inner_ref: Rc<RefCell<SplitInner<T, B>>>,
}

// pub struct SplitSync<T: Signal> {
//     id: usize,
//     inner_ref: Arc<RwLock<SplitInner<T>>>,
// }
//

impl<T: Signal, B> SplitInner<T, B> {
    fn new(inner: T, buf: SpmcRingBuf<B>) -> Self {
        Self { inner, buf }
    }
}

impl<T: Signal, B: Deref<Target = [T::Sample]>> Split<T, B> {
    pub fn new_buffered(inner: T, buf: B) -> Self {
        let spec = *inner.spec();
        let n_channels = spec.channels.count() as usize;

        let mut ring_buf = SpmcRingBuf::new(buf, n_channels);
        let id = ring_buf.add_instance();

        let split_inner = SplitInner::new(inner, ring_buf);
        let inner_ref = Rc::new(RefCell::new(split_inner));

        Self {
            id,
            spec,
            inner_ref,
        }
    }

    pub fn new(inner: T) -> Self
    where
        B: Default,
    {
        Self::new_buffered(inner, B::default())
    }
}

impl<T: Signal, B> Signal for Split<T, B> {
    type Sample = T::Sample;

    fn spec(&self) -> &SignalSpec {
        &self.spec
    }
}

impl<T, B> IndexedSignal for Split<T, B>
where
    T: IndexedSignal,
    B: Deref<Target = [T::Sample]>,
{
    fn pos(&self) -> u64 {
        let inner = self.inner_ref.as_ref().borrow();
        let offset = inner.buf.instance_remainder(&self.id) as u64;

        inner.inner.pos() - offset
    }
}

impl<T, B> FiniteSignal for Split<T, B>
where
    T: FiniteSignal,
{
    fn len(&self) -> u64 {
        self.inner_ref.as_ref().borrow().inner.len()
    }
}

// impl<T, B> SignalReader for Split<T, B>
// where
//     T: SignalReader,
//     B: DerefMut<Target = [T::Sample]>,
// {
//     fn read(&mut self, buf: &mut [Self::Sample]) -> Result<usize, PhonicError> {
//         let mut inner = self.inner_ref.as_ref().borrow_mut();
//         let cursor = inner.cursor.get(&self.id).ok_or(PhonicError::MissingData)?;
//         if cursor.empty || inner.empty {
//             inner.fill_buf()?;
//         }
//
//         let (trailing, leading) = inner.buf.instance_buf(&self.id);
//
//         let n_trailing = buf.len().min(trailing.len());
//         buf[..n_trailing].copy_from_slice(&trailing[..n_trailing]);
//
//         let n_leading = (buf.len() - n_trailing).min(leading.len());
//         let n_samples = n_trailing + n_leading;
//         if n_leading > 0 {
//             buf[n_trailing..n_samples].copy_from_slice(&leading[..n_leading]);
//         }
//
//         inner.buf.advance_instance(&self.id, n_samples)?;
//         Ok(n_samples)
//     }
// }

impl<T: Signal, B> Clone for Split<T, B> {
    fn clone(&self) -> Self {
        let id = self
            .inner_ref
            .as_ref()
            .borrow_mut()
            .buf
            .clone_instance(&self.id);

        Self {
            id,
            spec: self.spec,
            inner_ref: self.inner_ref.clone(),
        }
    }
}

impl<T: Signal, B> Drop for Split<T, B> {
    fn drop(&mut self) {
        self.inner_ref
            .as_ref()
            .borrow_mut()
            .buf
            .remove_instance(&self.id);
    }
}
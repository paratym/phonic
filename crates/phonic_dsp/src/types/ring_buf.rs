use phonic_core::PhonicError;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy)]
struct Cursor {
    pos: usize,
    empty: bool,
}

pub struct SpmcRingBuf<B> {
    buf: B,
    buf_len: usize,
    start: usize,
    end: usize,
    empty: bool,

    next_id: AtomicUsize,
    cursor: HashMap<usize, Cursor>,
}

impl<B> SpmcRingBuf<B> {
    pub fn new<S>(buf: B, align: usize) -> Self
    where
        B: Deref<Target = [S]>,
    {
        let mut buf_len = buf.len();
        buf_len -= buf_len % align;

        SpmcRingBuf {
            buf,
            buf_len,
            start: 0,
            end: 0,
            empty: true,
            next_id: AtomicUsize::default(),
            cursor: HashMap::default(),
        }
    }

    pub fn add_instance(&mut self) -> usize {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let cursor = Cursor {
            pos: self.start,
            empty: self.empty,
        };

        self.cursor.insert(id, cursor);

        id
    }

    pub fn clone_instance(&mut self, id: &usize) -> usize {
        let Some(cursor) = self.cursor.get(id).cloned() else {
            return self.add_instance();
        };

        let clone_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.cursor.insert(clone_id, cursor);

        clone_id
    }

    pub fn remove_instance(&mut self, id: &usize) {
        self.cursor.remove(id);
    }

    fn buf_distance(&self, start: usize, end: usize) -> usize {
        (end + self.buf_len - start) % self.buf_len
    }

    pub fn trim_buf(&mut self) -> usize {
        let start_offset = self
            .cursor
            .values()
            .map(|cursor| (cursor, self.buf_distance(self.start, cursor.pos)))
            .min_by_key(|(_, offset)| *offset);

        let Some((start, offset)) = start_offset else {
            return 0;
        };

        self.start = start.pos;
        self.empty |= offset > 0 && self.start == self.end;

        offset
    }

    pub fn available_buf<S>(&mut self) -> &mut [S]
    where
        B: DerefMut<Target = [S]>,
    {
        let buf_wraps = self.start > self.end || (self.start == self.end && !self.empty);
        let end = if buf_wraps { self.start } else { self.buf_len };

        &mut self.buf[self.end..end]
    }

    pub fn instance_remainder(&self, id: &usize) -> usize {
        let Some(cursor) = self.cursor.get(id) else {
            return 0;
        };

        self.buf_distance(cursor.pos, self.end)
    }

    pub fn extend_buf(&mut self, n: usize) -> Result<(), PhonicError> {
        let available = if self.empty {
            self.buf_len
        } else {
            self.buf_distance(self.end, self.start)
        };

        if n > available {
            return Err(PhonicError::OutOfBounds);
        }

        self.end += n;
        self.end %= self.buf_len;
        self.empty &= n == 0;
        self.cursor
            .values_mut()
            .for_each(|cursor| cursor.empty &= n == 0);

        Ok(())
    }

    pub fn instance_buf<S>(&self, id: &usize) -> (&[S], &[S])
    where
        B: Deref<Target = [S]>,
    {
        let Some(cursor) = self.cursor.get(id) else {
            let empty = &self.buf[self.end..self.end];
            return (empty, empty);
        };

        let buf_wraps = self.end > cursor.pos
            || (self.start == self.end && cursor.pos == self.start && !cursor.empty);

        if !buf_wraps {
            return (
                &self.buf[cursor.pos..self.end],
                &self.buf[self.end..self.end],
            );
        }

        (&self.buf[cursor.pos..self.buf_len], &self.buf[..self.end])
    }

    pub fn advance_instance(&mut self, id: &usize, n: usize) -> Result<(), PhonicError> {
        let Some(cursor) = self.cursor.get_mut(id) else {
            return Err(PhonicError::NotFound);
        };

        let buf_wraps = self.end > cursor.pos
            || (self.start == self.end && cursor.pos == self.start && !cursor.empty);

        let available = if !buf_wraps {
            self.end - cursor.pos
        } else {
            self.buf_len - cursor.pos + self.end
        };

        if n > available {
            return Err(PhonicError::OutOfBounds);
        }

        cursor.pos += n;
        cursor.pos %= self.buf_len;
        cursor.empty |= cursor.pos == self.end;

        Ok(())
    }
}

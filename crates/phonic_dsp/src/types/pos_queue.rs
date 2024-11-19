use crate::types::IndexedSignalList;
use std::collections::LinkedList;

#[derive(Clone, Copy)]
pub struct PosCursor {
    pub id: usize,
    pub pos: u64,
}

pub struct PosQueue {
    queue: LinkedList<PosCursor>,
}

impl PosQueue {
    pub fn new<T: IndexedSignalList>(list: &T) -> Self {
        let range = 0..list.count();
        let mut positions = range
            .map(|id| (id, list.pos(id)))
            .map(|(id, pos)| PosCursor { id, pos })
            .collect::<Box<[_]>>();

        positions.sort_unstable_by_key(|cursor| cursor.pos);
        let queue = positions.iter().copied().collect();

        Self { queue }
    }

    pub fn insert(&mut self, cursor: PosCursor) {
        let mut tail = LinkedList::new();

        loop {
            let Some(cmp_cursor) = self.queue.pop_back() else {
                break;
            };

            if cmp_cursor.pos > cursor.pos {
                tail.push_front(cmp_cursor)
            } else {
                self.queue.push_back(cmp_cursor);
                break;
            }
        }

        self.queue.push_back(cursor);
        self.queue.append(&mut tail);
    }

    pub fn peek_front(&self) -> Option<&PosCursor> {
        self.queue.front()
    }

    pub fn pop_front(&mut self) -> Option<PosCursor> {
        self.queue.pop_front()
    }

    pub fn commit_front(&mut self, n_frames: u64) {
        let Some(mut cursor) = self.queue.pop_front() else {
            return;
        };

        cursor.pos += n_frames;
        self.insert(cursor)
    }
}

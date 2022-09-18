use crate::helper::copy;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct Buffer<T> {
    blocks: VecDeque<Vec<T>>,
}

impl<T> Buffer<T> {
    pub fn new() -> Self {
        Self {
            blocks: Default::default(),
        }
    }
    /// append a chunk of data into the buffer
    pub fn push(&mut self, block: Vec<T>) {
        self.blocks.push_back(block);
    }
    /// fetch a chunk of data into the buffer
    pub fn pop(&mut self) -> Option<Vec<T>> {
        self.blocks.pop_front()
    }

    /// append a chunk of data, with iterator
    pub fn push_iter<I>(&mut self, src: I)
    where
        I: Iterator<Item = T>,
    {
        self.push(src.collect());
    }
}

impl<T: Clone> Buffer<T> {
    /// append a chunk of data, with iterator
    pub fn push_iter_ref<'a, I>(&mut self, src: I)
    where
        T: 'a,
        I: Iterator<Item = &'a T>,
    {
        self.push_iter(src.cloned());
    }

    /// append a chunk of data, with slice
    pub fn push_slice(&mut self, src: &[T]) {
        self.push_iter_ref(src.iter());
    }

    /// fetch a chunk of data to fill the slice.
    /// return the number of poped elements.
    pub fn pop_slice(&mut self, dest: &mut [T]) -> usize {
        let n = dest.len();

        let mut i = 0;
        while i < n {
            if let Some(mut block) = self.blocks.pop_front() {
                let m = std::cmp::min(n - i, block.len());
                copy(dest[i..].iter_mut(), block.drain(..m));
                i += m;
                if !block.is_empty() {
                    self.blocks.push_front(block);
                }
            } else {
                break;
            }
        }

        i
    }
}

impl<T> Default for Buffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::{Arc, Mutex, MutexGuard};
#[derive(Clone, Debug)]
pub struct ConcurrentBuffer<T>(Arc<Mutex<Buffer<T>>>);

impl<T> ConcurrentBuffer<T> {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(Buffer::new())))
    }
    pub fn lock(&self) -> MutexGuard<'_, Buffer<T>> {
        self.0.lock().expect("ConcurrentBuffer mutex is poisonous")
    }
}

impl<T> Default for ConcurrentBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

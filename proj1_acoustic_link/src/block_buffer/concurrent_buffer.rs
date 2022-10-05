use super::Buffer;
use crate::traits::{InStream, OutStream};
use std::{
  sync::{Arc, Mutex},
  thread,
};
#[derive(Clone, Debug)]
/// Thread-safe wrapper of [`Buffer`].  
/// Implemented with shared memory with mutex lock ([`Arc`] of [`Mutex`]).
pub struct ConcurrentBuffer<T>(Arc<Mutex<Buffer<T>>>);

impl<T> ConcurrentBuffer<T> {
  pub fn new() -> Self {
    Self(Arc::new(Mutex::new(Buffer::new())))
  }

  /// Clear the buffer
  pub fn clear(&self) {
    let mut this = self.0.lock().unwrap();
    this.clear()
  }
}

impl<T> Default for ConcurrentBuffer<T> {
  fn default() -> Self {
    Self::new()
  }
}

impl<T: Clone> InStream<T, ()> for Buffer<T> {
  fn read(&mut self, buf: &mut [T]) -> Result<usize, ()> {
    Ok(self.pop_slice(buf))
  }

  fn read_exact(&mut self, buf: &mut [T]) -> Result<(), ()> {
    let mut n = 0;
    while n < buf.len() {
      if let Ok(m) = self.read(&mut buf[n..]) {
        n += m;
      }
    }
    Ok(())
  }
}

impl<T: Clone> OutStream<T, ()> for Buffer<T> {
  fn write(&mut self, buf: &[T]) -> Result<usize, ()> {
    self.push_slice(buf);
    Ok(buf.len())
  }

  fn write_exact(&mut self, buf: &[T]) -> Result<(), ()> {
    self.push_slice(buf);
    Ok(())
  }
}

impl<T: Clone> InStream<T, ()> for ConcurrentBuffer<T> {
  fn read(&mut self, buf: &mut [T]) -> Result<usize, ()> {
    let mut this = self.0.lock().unwrap();
    this.read(buf)
  }

  fn read_exact(&mut self, buf: &mut [T]) -> Result<(), ()> {
    let mut n = 0;
    while n < buf.len() {
      if let Ok(mut this) = self.0.lock() {
        if let Ok(m) = this.read(buf) {
          n += m;
        }
      } else {
        return Err(());
      }
      thread::yield_now();
    }
    Ok(())
  }
}
impl<T: Clone> OutStream<T, ()> for ConcurrentBuffer<T> {
  fn write(&mut self, buf: &[T]) -> Result<usize, ()> {
    let mut this = self.0.lock().unwrap();
    this.write(buf)
  }

  fn write_exact(&mut self, buf: &[T]) -> Result<(), ()> {
    let mut this = self.0.lock().unwrap();
    this.write_exact(buf)
  }
}

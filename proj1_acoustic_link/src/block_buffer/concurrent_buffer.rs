use super::Buffer;
use crate::traits::{InStream, OutStream};
use parking_lot::{Condvar, Mutex};
use std::{sync::Arc, thread};

#[derive(Clone, Debug)]
/// Thread-safe wrapper of [`Buffer`].  
/// Implemented with shared memory with mutex lock ([`Arc`] of [`Mutex`]).
pub struct ConcurrentBuffer<T>(Arc<(Mutex<Buffer<T>>, Condvar)>);

impl<T> ConcurrentBuffer<T> {
  pub fn new() -> Self {
    Self(Arc::new((Mutex::new(Buffer::new()), Condvar::new())))
  }

  /// Clear the buffer
  pub fn clear(&mut self) {
    let &(ref lock, ref cvar) = &*self.0;
    let mut buf = lock.lock();
    buf.clear();
    cvar.notify_all();
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

  /// `Buffer` is a single-threaded buffer
  /// no other thread can empty the buffer
  fn wait(&mut self) {
    unimplemented!()
  }
}

impl<T: Clone> InStream<T, ()> for ConcurrentBuffer<T> {
  fn read(&mut self, dest: &mut [T]) -> Result<usize, ()> {
    let &(ref lock, ref cvar) = &*self.0;
    let mut buf = lock.lock();
    let n = buf.read(dest)?;
    if buf.empty() {
      cvar.notify_all();
    }
    Ok(n)
  }

  fn read_exact(&mut self, dest: &mut [T]) -> Result<(), ()> {
    let mut n = 0;
    while n < dest.len() {
      let &(ref lock, _) = &*self.0;
      let mut buf = lock.lock();
      if let Ok(m) = buf.read(dest) {
        n += m;
      }
      thread::yield_now();
    }
    Ok(())
  }
}
impl<T: Clone> OutStream<T, ()> for ConcurrentBuffer<T> {
  fn write(&mut self, src: &[T]) -> Result<usize, ()> {
    let &(ref lock, _) = &*self.0;
    let mut buf = lock.lock();
    buf.write(src)
  }

  fn write_exact(&mut self, src: &[T]) -> Result<(), ()> {
    let &(ref lock, _) = &*self.0;
    let mut buf = lock.lock();
    buf.write_exact(src)
  }

  /// wait other thread to empty the concurrent buffer
  fn wait(&mut self) {
    let &(ref lock, ref cvar) = &*self.0;
    let mut buf = lock.lock();
    cvar.wait_while(&mut buf, |buf| !buf.empty());
  }
}

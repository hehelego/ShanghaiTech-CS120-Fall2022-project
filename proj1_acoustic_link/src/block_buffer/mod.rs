mod buffer;
use std::thread;

pub use buffer::{Buffer, ConcurrentBuffer};

use crate::traits::{InStream, OutStream};

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
        let mut this = self.lock();
        this.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [T]) -> Result<(), ()> {
        let mut n = 0;
        while n < buf.len() {
            let mut this = self.lock();
            if let Ok(m) = this.read(buf) {
                n += m;
            }
            thread::yield_now();
        }
        Ok(())
    }
}
impl<T: Clone> OutStream<T, ()> for ConcurrentBuffer<T> {
    fn write(&mut self, buf: &[T]) -> Result<usize, ()> {
        let mut this = self.lock();
        this.write(buf)
    }

    fn write_exact(&mut self, buf: &[T]) -> Result<(), ()> {
        let mut this = self.lock();
        this.write_exact(buf)
    }
}

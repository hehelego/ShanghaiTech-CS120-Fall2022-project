use crate::{
  block_buffer::ConcurrentBuffer,
  traits::{InStream, OutStream},
};

/// A loopback streae.
/// The stream can read out whatever written into it.
#[derive(Clone)]
pub struct LoopBackStream(ConcurrentBuffer<f32>);

impl LoopBackStream {
  pub fn new() -> Self {
    Self(ConcurrentBuffer::new())
  }
}

impl Default for LoopBackStream {
  fn default() -> Self {
    Self::new()
  }
}

impl InStream<f32, ()> for LoopBackStream {
  fn read(&mut self, buf: &mut [f32]) -> Result<usize, ()> {
    self.0.read(buf)
  }

  fn read_exact(&mut self, buf: &mut [f32]) -> Result<(), ()> {
    self.0.read_exact(buf)
  }
}

impl OutStream<f32, ()> for LoopBackStream {
  fn write(&mut self, buf: &[f32]) -> Result<usize, ()> {
    self.0.write(buf)
  }

  fn write_exact(&mut self, buf: &[f32]) -> Result<(), ()> {
    self.0.write_exact(buf)
  }
}

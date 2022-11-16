use crate::traits::FP;
use crate::{
  block_buffer::ConcurrentBuffer,
  traits::{InStream, OutStream},
};

/// A loopback streae.
/// The stream can read out whatever written into it.
#[derive(Clone)]
pub struct LoopBackStream(ConcurrentBuffer<FP>);

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

impl InStream<FP, ()> for LoopBackStream {
  fn read(&mut self, buf: &mut [FP]) -> Result<usize, ()> {
    self.0.read(buf)
  }

  fn read_exact(&mut self, buf: &mut [FP]) -> Result<(), ()> {
    self.0.read_exact(buf)
  }
}

impl OutStream<FP, ()> for LoopBackStream {
  fn write(&mut self, buf: &[FP]) -> Result<usize, ()> {
    self.0.write(buf)
  }

  fn write_exact(&mut self, buf: &[FP]) -> Result<(), ()> {
    self.0.write_exact(buf)
  }
  /// wait for other thread to extract the samples
  fn wait(&mut self) {
    self.0.wait()
  }
}

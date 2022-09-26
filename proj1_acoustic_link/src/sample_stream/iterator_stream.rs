use crate::{helper::copy, traits::InStream};

/// A stream object continuously fetch data from an iterator.  
/// Majorly used for testing.
pub struct IteratorInStream<I>(I);

impl<I> IteratorInStream<I> {
  /// create a stream from an iterator
  pub fn new(iter: I) -> Self {
    Self(iter)
  }
}
impl<F> IteratorInStream<std::iter::FromFn<F>>
where
  F: FnMut() -> Option<f32>,
{
  /// create a input stream from a function that can generate f32 value
  pub fn from_fn(func: F) -> Self {
    Self::new(std::iter::from_fn(func))
  }
}

impl<I> InStream<f32, ()> for IteratorInStream<I>
where
  I: Iterator<Item = f32>,
{
  fn read(&mut self, buf: &mut [f32]) -> Result<usize, ()> {
    Ok(copy(buf.iter_mut(), self.0.by_ref()))
  }

  fn read_exact(&mut self, buf: &mut [f32]) -> Result<(), ()> {
    self.read(buf).map(|_| ())
  }
}

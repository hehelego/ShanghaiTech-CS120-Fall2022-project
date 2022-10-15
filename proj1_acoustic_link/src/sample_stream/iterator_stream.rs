use crate::traits::FP;
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
  F: FnMut() -> Option<FP>,
{
  /// create a input stream from a function that can generate FP value
  pub fn from_fn(func: F) -> Self {
    Self::new(std::iter::from_fn(func))
  }
}

impl<I> InStream<FP, ()> for IteratorInStream<I>
where
  I: Iterator<Item = FP>,
{
  fn read(&mut self, buf: &mut [FP]) -> Result<usize, ()> {
    Ok(copy(buf.iter_mut(), self.0.by_ref()))
  }

  fn read_exact(&mut self, buf: &mut [FP]) -> Result<(), ()> {
    self.read(buf).map(|_| ())
  }
}

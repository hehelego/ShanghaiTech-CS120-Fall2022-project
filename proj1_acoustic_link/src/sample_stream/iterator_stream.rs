use crate::traits::{InStream, OutStream};

/// A stream object continuously fetch data from an iterator
pub struct IteratorInStream<I> {
    iter: I,
}

impl<I> IteratorInStream<I> {
    /// create a stream from an iterator
    pub fn new(iter: I) -> Self {
        Self { iter }
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
        todo!()
    }

    fn read_exact(&mut self, buf: &mut [f32]) -> Result<(), ()> {
        todo!()
    }
}

/// A stream object continuously write data into an iterator
pub struct IteratorOutStream<I> {
    iter: I,
}

impl<I> IteratorOutStream<I> {
    /// create a stream from an iterator
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}
impl<'a, F> IteratorOutStream<std::iter::FromFn<F>>
where
    F: FnMut() -> Option<&'a mut f32>,
{
    /// create a input stream from a function that can generate f32 mutable reference
    pub fn from_fn(func: F) -> Self {
        Self::new(std::iter::from_fn(func))
    }
}

impl<'a, I> OutStream<f32, ()> for IteratorOutStream<I>
where
    I: Iterator<Item = &'a mut f32>,
{
    fn write(&mut self, buf: &[f32]) -> Result<usize, ()> {
        todo!()
    }

    fn write_exact(&mut self, buf: &[f32]) -> Result<(), ()> {
        todo!()
    }
}

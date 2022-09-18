use crate::traits::{InStream, OutStream};

pub struct IteratorInStream<I> {
    iter: I,
}

impl<I> IteratorInStream<I> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}
impl<F> IteratorInStream<std::iter::FromFn<F>>
where
    F: FnMut() -> Option<f32>,
{
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

pub struct IteratorOutStream<I> {
    iter: I,
}

impl<I> IteratorOutStream<I> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}
impl<'a, F> IteratorOutStream<std::iter::FromFn<F>>
where
    F: FnMut() -> Option<&'a mut f32>,
{
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

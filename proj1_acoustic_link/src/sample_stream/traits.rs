use crate::traits::{InStream, OutStream};

/// An input stream where the streaming data type is f32.
pub trait SampleInStream<E>: InStream<f32, E> {}
impl<S, E> SampleInStream<E> for S where S: InStream<f32, E> {}

/// An output stream where the streaming data type is f32.
pub trait SampleOutStream<E>: OutStream<f32, E> {}
impl<S, E> SampleOutStream<E> for S where S: OutStream<f32, E> {}

use crate::traits::{InStream, IoStream, OutStream};

/// An input stream where the streaming data type is f32.
pub trait SampleInStream<E>: InStream<f32, E> {}
impl<S, E> SampleInStream<E> for S where S: InStream<f32, E> {}

/// An output stream where the streaming data type is f32.
pub trait SampleOutStream<E>: OutStream<f32, E> {}
impl<S, E> SampleOutStream<E> for S where S: OutStream<f32, E> {}

/// A stream where the streaming data type is f32.
pub trait SampleIoStream<E>: IoStream<f32, E> {}
impl<S, E> SampleIoStream<E> for S where S: IoStream<f32, E> {}

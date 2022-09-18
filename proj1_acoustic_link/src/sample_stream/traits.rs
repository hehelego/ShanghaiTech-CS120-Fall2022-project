use crate::traits::{InStream, IoStream, OutStream};

pub trait SampleInStream<E>: InStream<f32, E> {}
impl<S, E> SampleInStream<E> for S where S: InStream<f32, E> {}

pub trait SampleOutStream<E>: OutStream<f32, E> {}
impl<S, E> SampleOutStream<E> for S where S: OutStream<f32, E> {}

pub trait SampleIoStream<E>: IoStream<f32, E> {}
impl<S, E> SampleIoStream<E> for S where S: IoStream<f32, E> {}

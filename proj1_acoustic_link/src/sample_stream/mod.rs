/// type trait alias
mod traits;
pub use traits::{SampleInStream, SampleIoStream, SampleOutStream};

/// sample stream IO with cpal audio I/O
mod cpal_stream;
/// sample stream IO with hound wav file IO
mod file_stream;
/// sample stream IO with iterators
mod iterator_stream;
/// sample stream IO with a concurrent buffer, read out the written samples
mod loopback_stream;

pub use cpal_stream::{CpalInStream, CpalOutStream};
pub use iterator_stream::{IteratorInStream, IteratorOutStream};
pub use loopback_stream::LoopBackStream;

#[cfg(test)]
mod tests;

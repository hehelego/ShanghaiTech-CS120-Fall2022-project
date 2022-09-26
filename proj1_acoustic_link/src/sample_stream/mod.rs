/// type trait alias
mod traits;
pub use traits::{SampleInStream, SampleOutStream};

/// sample stream IO with cpal audio I/O
mod cpal_stream;
/// sample stream IO with hound wav reader/writer
mod hound_stream;
/// sample stream IO with iterators
mod iterator_stream;
/// sample stream IO with a concurrent buffer, read out the written samples
mod loopback_stream;

pub use cpal_stream::{CpalInStream, CpalOutStream};
pub use hound_stream::{HoundInStream, HoundOutStream};
pub use iterator_stream::IteratorInStream;
pub use loopback_stream::LoopBackStream;

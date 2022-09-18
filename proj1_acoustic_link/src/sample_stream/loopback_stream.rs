use crate::block_buffer::ConcurrentBuffer;

/// A loopback streae.
/// The stream can read out whatever written into it.
pub type LoopBackStream = ConcurrentBuffer<f32>;

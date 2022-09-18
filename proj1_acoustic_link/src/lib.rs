/// common helper functions
pub mod helper;

/// define [`IoStream`] and [`PacketTxRx`] traits.
pub mod traits;

/// implementors of [`PacketTxRx`], where a packet is a physics layer frame.
pub mod phy_packet;

/// implementors of [`IoStream`], where a the stream data is of type f32.
pub mod sample_stream;

/// blockwise buffer and its thread safe wrapper
/// [`block_buffer::Buffer`] implements [`IoStream`]
/// [`block_buffer::BlockBuffer`] implements [`IoStream`]
pub mod block_buffer;

/// common helper functions
pub mod helper;

/// defines [`traits::InStream`]/[`traits::OutStream`] and
/// [`traits::PacketSender`]/[`traits::PacketReceiver`] traits.
pub mod traits;

/// implementors of [`traits::PacketSender`] and [`traits::PacketReceiver`],
/// where the packet is a physics layer packet, a fixed size byte slice.
pub mod phy_packet;

/// implementors of [`traits::InStream`] and [`traits::OutStream`],
/// where the stream data type is floating point 32bit PCM sample.
pub mod sample_stream;

/// blockwise buffer and its thread safe wrapper.
pub mod block_buffer;

pub mod phy_layer;

// Configurations for the audio stream
mod default_config;
pub use default_config::DefaultConfig;

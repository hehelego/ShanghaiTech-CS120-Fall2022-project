/// Frame: a audio signal.
/// A frame consists of a preamble section and a payload section.
///
/// Packet: a chunk of bytes.
/// A packet is sent/received with no data integrity guarantee

/// define the types and traits related to physisc layer packet
pub mod traits;
pub use traits::{Codec, FrameDetector, FramePayload, PhyPacket, PreambleGen};

/// implementors of [`FrameDetector`]: audio stream framing algorithms.
pub mod frame_detect;
/// implementors of [`Codec`]: modulation methods.
pub mod modulation;
/// implementors of [`PreambleGen`]: preamble sequences.
pub mod preambles;

/// Bytes packet (packet type [`PhyPacket`]) transmission on audio PCM sample streams.  
/// A sender can be built on a stream with a [`PreambleGen`] and a [`Codec`].  
/// A receiver can be built on a stream with a [`PreambleGen`], a [`FrameDetector`] and a [`Codec`].  
pub mod txrx;

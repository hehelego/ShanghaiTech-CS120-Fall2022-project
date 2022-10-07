/// Frame: a audio signal.
/// A frame consists of a preamble section and a payload section.
///
/// Packet: a chunk of bytes.
/// A packet is sent/received with no data integrity guarantee

/// define the traits for physics layer
pub mod traits;
pub use traits::{Codec, FramePayload, FrameDetector, PhyPacket, PhyPacketReceiver, PhyPacketSender, PreambleGen};

/// implementors of [`FrameDetector`]: audio stream framing algorithms.
pub mod frame_detect;
/// implementors of [`Codec`]: modulation methods.
pub mod modulation;
/// implementors of [`PreambleGen`]: preamble sequences.
pub mod preambles;

/// A systematic implementation of the physisc layer on audio PCM sample streams.  
/// A [`PreambleGen`], a [`Codec`] and a [`FrameDetector`] together defines a PHY layer.  
/// Implementors of [`PhyPacketSender`] and [`PhyPacketReceiver`] are provided.
pub mod audio_phy_txrx;

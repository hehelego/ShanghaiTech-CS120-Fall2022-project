/// define the traits for physics layer
pub mod traits;
pub use traits::{
  Codec, Frame, FrameDetector, PhyPacket, PhyPacketReceiver, PhyPacketSender, PhyPacketTxRx, PreambleGenerator,
};

/// implementors of [`FrameDetector`]: audio stream framing algorithms.
pub mod frame_detect;
/// implementors of [`Codec`]: modulation methods.
pub mod modulation;
/// implementors of [`PreambleGenerator`]: preamble sequences.
pub mod preambles;

/// A systematic implementation of the physisc layer on audio PCM sample streams.  
/// A [`PreambleGenerator`], a [`Codec`] and a [`FrameDetector`] together defines a [`PhyScheme`].  
/// Implementors of [`PhyPacketSender`], [`PhyPacketReceiver`] and [`PhyPacketTxRx`] are provided.
pub mod audio_phy_txrx;

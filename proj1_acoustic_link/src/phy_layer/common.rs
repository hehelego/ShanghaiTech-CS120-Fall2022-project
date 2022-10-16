pub use crate::phy_packet::{
  frame_detect::CorrelationFraming as FrameDetector, modem::PSK, preambles::ChirpUpDown as Preamble, txrx::PhyReceiver,
  txrx::PhySender,
};

pub use crate::phy_packet::{Modem, PhyPacket, PreambleGen};
pub use crate::sample_stream::{CpalInStream as InStream, CpalOutStream as OutStream};
pub use crate::traits::{PacketReceiver, PacketSender};

// physice packet sender type
pub type Tx = PhySender<Preamble, PSK, OutStream, ()>;
// physice packet receiver type
pub type Rx = PhyReceiver<Preamble, PSK, FrameDetector<Preamble>, InStream, ()>;

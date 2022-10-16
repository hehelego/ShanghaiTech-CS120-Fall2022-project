pub use crate::phy_packet::{
  frame_detect::CorrelationFraming as FrameDetector, modem::OFDM as ModemMethod, preambles::ChirpUpDown as Preamble,
  txrx::PhyReceiver, txrx::PhySender,
};

pub use crate::sample_stream::{CpalInStream as InStream, CpalOutStream as OutStream};

// physice packet sender type
pub type Tx = PhySender<Preamble, ModemMethod, OutStream, ()>;
// physice packet receiver type
pub type Rx = PhyReceiver<Preamble, ModemMethod, FrameDetector<Preamble>, InStream, ()>;

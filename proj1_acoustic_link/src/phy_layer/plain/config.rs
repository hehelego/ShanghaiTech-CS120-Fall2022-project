#[cfg(feature = "wired")]
pub use crate::phy_packet::modem::LineCode as ModemMethod;
#[cfg(not(feature = "wired"))]
pub use crate::phy_packet::modem::PSK as ModemMethod;

pub use crate::phy_packet::{
  frame_detect::CorrelationFraming as FrameDetector, preambles::ChirpUpDown as Preamble, txrx::PhyReceiver,
  txrx::PhySender,
};
use std::time::Duration;

pub use crate::sample_stream::{CpalInStream as InStream, CpalOutStream as OutStream, CpalPowerProbe as PowerProbe};

/// physice packet sender type
pub type Tx = PhySender<Preamble, ModemMethod, OutStream, ()>;
/// physice packet receiver type
pub type Rx = PhyReceiver<Preamble, ModemMethod, FrameDetector<Preamble>, InStream, ()>;

/// the channel is considered free when the power is smaller than [`REST_POWER`]
pub const REST_POWER: f32 = if cfg!(feature = "wired") { 1e-5 } else { 1e9 };
pub const ESTIMATED_RTT: Duration = Duration::from_millis(150);

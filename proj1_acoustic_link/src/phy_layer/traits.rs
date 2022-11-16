use std::time::Duration;

pub use crate::phy_packet::PhyPacket;
pub use crate::traits::{PacketReceiver, PacketSender};

/// The PHY layer service provider trait:
/// Sending/receiving packets of fixed bytes with no correctness or delivery guarantee
///
/// - `SendErr`: the error type that may occur when sending a packet
/// - `RecvErr`: the error type that may occur when receiving a packet
pub trait PhyLayer<SendErr, RecvErr>: PacketSender<PhyPacket, SendErr> + PacketReceiver<PhyPacket, RecvErr> {
  /// number of bytes in one packet
  const PACKET_BYTES: usize;
  /// estimated RTT on the channel
  const ESTIMATED_RTT: Duration;

  /// Determine if the channel is free so that we can send a packet
  fn channel_free(&self) -> bool;
}

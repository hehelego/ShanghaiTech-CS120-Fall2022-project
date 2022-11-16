use std::time::Duration;

use super::PhyLayer;
pub use crate::phy_packet::{Modem, PhyPacket, PreambleGen};
pub use crate::traits::{PacketReceiver, PacketSender};
use config::*;

/// a physics layer peer object.
/// send/recv packets with no latency/correctness guarantee.
pub struct PlainPHY {
  tx: Tx,
  rx: Rx,
  power_probe: PowerProbe,
}

impl PlainPHY {
  /// number of samples in one packet
  pub const PACKET_SAMPLES: usize = Tx::SAMPLES_PER_PACKET;

  /// combine a sender and a receiver to get a physics layer object
  pub fn new(tx: Tx, rx: Rx, power_probe: PowerProbe) -> Self {
    Self { tx, rx, power_probe }
  }
}

impl PhyLayer for PlainPHY {
  type SendErr = ();
  type RecvErr = ();
  const PACKET_BYTES: usize = ModemMethod::BYTES_PER_PACKET;
  const ESTIMATED_RTT: Duration = ESTIMATED_RTT;

  fn channel_free(&self) -> bool {
    self.power_probe.power() < REST_POWER
  }
}

impl PacketSender<PhyPacket, ()> for PlainPHY {
  /// send a packet, return until send finished or error
  fn send(&mut self, packet: PhyPacket) -> Result<(), ()> {
    assert_eq!(packet.len(), Self::PACKET_BYTES);
    self.tx.send(packet)
  }
}
impl PacketReceiver<PhyPacket, ()> for PlainPHY {
  /// receive a packet, return received a packet or error
  fn recv(&mut self) -> Result<PhyPacket, ()> {
    self.rx.recv()
  }

  fn recv_timeout(&mut self, timeout: std::time::Duration) -> Result<PhyPacket, ()> {
    self.rx.recv_timeout(timeout)
  }

  fn recv_peek(&mut self) -> bool {
    self.rx.recv_peek()
  }
}

impl Default for PlainPHY {
  fn default() -> Self {
    let tx = Tx::new(OutStream::default(), ModemMethod::default());
    let rx = Rx::new(
      InStream::default(),
      ModemMethod::default(),
      FrameDetector::new::<{ ModemMethod::SAMPLES_PER_PACKET }>(Preamble::new()),
    );
    let power_probe = PowerProbe::default();
    Self::new(tx, rx, power_probe)
  }
}

mod config;

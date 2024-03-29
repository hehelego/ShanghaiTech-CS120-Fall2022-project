pub use crate::phy_packet::{Modem, PhyPacket, PreambleGen};
pub use crate::traits::{PacketReceiver, PacketSender};

use config::*;

/// a physics layer peer object.
/// use OFDM+BPSK for modulation.
/// similar to [`super::PlainPHY`], no correctness guarantee for transmission.
pub struct HighBpsPHY {
  tx: Tx,
  rx: Rx,
}

impl HighBpsPHY {
  /// number of bytes in one packet
  pub const PACKET_BYTES: usize = ModemMethod::BYTES_PER_PACKET;
  /// number of samples in one packet
  pub const PACKET_SAMPLES: usize = Tx::SAMPLES_PER_PACKET;

  /// combine a sender and a receiver to get a physics layer object
  pub fn new(tx: Tx, rx: Rx) -> Self {
    Self { tx, rx }
  }
}

impl PacketSender<PhyPacket, ()> for HighBpsPHY {
  /// send a packet, return until send finished or error
  fn send(&mut self, packet: PhyPacket) -> Result<(), ()> {
    assert_eq!(packet.len(), Self::PACKET_BYTES);
    self.tx.send(packet)
  }
}
impl PacketReceiver<PhyPacket, ()> for HighBpsPHY {
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

impl Default for HighBpsPHY {
  fn default() -> Self {
    let tx = Tx::new(OutStream::default(), ModemMethod::default());
    let rx = Rx::new(
      InStream::default(),
      ModemMethod::default(),
      FrameDetector::new::<{ ModemMethod::SAMPLES_PER_PACKET }>(Preamble::new()),
    );
    Self::new(tx, rx)
  }
}

mod config;

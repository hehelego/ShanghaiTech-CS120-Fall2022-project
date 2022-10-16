use super::common::*;

/// a physics layer peer object.
/// send/recv packets with no latency/correctness guarantee.
pub struct PlainPHY {
  tx: Tx,
  rx: Rx,
}

impl PlainPHY {
  /// number of bytes in one packet
  pub const PACKET_BYTES: usize = PSK::BYTES_PER_PACKET;
  /// number of samples in one packet
  pub const PACKET_SAMPLES: usize = Tx::SAMPLES_PER_PACKET;

  /// combine a sender and a receiver to get a physics layer object
  pub fn new(tx: Tx, rx: Rx) -> Self {
    Self { tx, rx }
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
}

impl Default for PlainPHY {
  fn default() -> Self {
    let tx = Tx::new(OutStream::default(), PSK::default());
    let rx = Rx::new(
      InStream::default(),
      PSK::default(),
      FrameDetector::new::<{ PSK::SAMPLES_PER_PACKET }>(Preamble::new()),
    );
    Self::new(tx, rx)
  }
}

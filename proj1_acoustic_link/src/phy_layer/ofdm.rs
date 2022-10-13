pub use crate::phy_packet::{
  frame_detect::CorrelationFraming as FrameDetector, modulation::OFDM as Codec_, preambles::ChirpUpDown as Preamble,
  txrx::PhyReceiver, txrx::PhySender,
};

pub use crate::phy_packet::{Codec, PhyPacket, PreambleGen};
pub use crate::sample_stream::{CpalInStream as InStream, CpalOutStream as OutStream};
pub use crate::traits::{PacketReceiver, PacketSender};

// physice packet sender type
pub type Tx = PhySender<Preamble, Codec_, OutStream, ()>;
// physice packet receiver type
pub type Rx = PhyReceiver<Preamble, Codec_, FrameDetector<Preamble>, InStream, ()>;

/// a physics layer peer object.
/// use OFDM+BPSK for modulation.
/// similar to [`super::PlainPHY`], no correctness guarantee for transmission.
pub struct HighBpsPHY {
  tx: Tx,
  rx: Rx,
}

impl HighBpsPHY {
  /// number of bytes in one packet
  pub const PACKET_BYTES: usize = Codec_::BYTES_PER_PACKET;
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
}

impl Default for HighBpsPHY {
  fn default() -> Self {
    let tx = Tx::new(OutStream::default(), Codec_::default());
    let rx = Rx::new(
      InStream::default(),
      Codec_::default(),
      FrameDetector::new::<{ Codec_::SAMPLES_PER_PACKET }>(Preamble::new()),
    );
    Self::new(tx, rx)
  }
}

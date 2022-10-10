use super::common::*;

type Tx = PhySender<Preamble, Codec_, OutStream, ()>;
type Rx = PhyReceiver<Preamble, Codec_, FrameDetector<Preamble>, InStream, ()>;

/// a physics layer peer object.
/// send/recv packets with no latency/correctness guarantee.
pub struct PlainPHY {
  tx: Tx,
  rx: Rx,
}

impl PlainPHY {
  /// number of bytes in one packet
  pub const PACKET_BYTES: usize = Codec_::BYTES_PER_PACKET;
  /// number of samples in one packet
  pub const PACKET_SAMPLES: usize = Preamble::PREAMBLE_LEN + Codec_::SAMPLES_PER_PACKET;

  pub fn new(tx: Tx, rx: Rx) -> Self {
    Self { tx, rx }
  }

  /// send a packet, return until send finished or error
  pub fn send(&mut self, packet: PhyPacket) -> Result<(), ()> {
    self.tx.send(packet)
  }
  /// receive a packet, return received a packet or error
  pub fn recv(&mut self) -> Result<PhyPacket, ()> {
    self.rx.recv()
  }
}

impl Default for PlainPHY {
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

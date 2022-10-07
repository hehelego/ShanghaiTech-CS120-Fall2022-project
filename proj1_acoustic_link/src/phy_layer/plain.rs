use crate::phy_packet::{
  audio_phy_txrx::PhyReceiver, audio_phy_txrx::PhySender, frame_detect::CorrelationFraming as FrameDetector,
  modulation::PSK as Codec_, preambles::ChirpUpDown as Preamble,
};
use crate::phy_packet::{PhyPacket, Codec, PreambleGen};

use crate::sample_stream::{CpalInStream as InStream, CpalOutStream as OutStream};
use crate::traits::{PacketReceiver, PacketSender};

type Tx = PhySender<Preamble, Codec_, OutStream, ()>;
type Rx = PhyReceiver<Preamble, Codec_, FrameDetector, InStream, ()>;

/// a physics layer peer object.
/// send/recv packets with no latency/correctness guarantee.
pub struct Plain {
  tx: Tx,
  rx: Rx,
}

impl Plain {
  pub const PACKET_BYTES: usize = Codec_::BYTES_PER_PACKET;
  pub const PACKET_SAMPLES: usize = Preamble::PREAMBLE_LEN + Codec_::SAMPLES_PER_PACKET;
  pub fn new(tx: Tx, rx: Rx) -> Self {
    Self { tx, rx }
  }
}

impl PacketSender<PhyPacket, ()> for Plain {
  fn send(&mut self, packet: PhyPacket) -> Result<(), ()> {
    self.tx.send(packet)
  }
}
impl PacketReceiver<PhyPacket, ()> for Plain {
  fn recv(&mut self) -> Result<PhyPacket, ()> {
    self.rx.recv()
  }
}

impl Default for Plain {
  fn default() -> Self {
    let tx = Tx::new(OutStream::default(), Codec_::default());
    let rx = Rx::new(
      InStream::default(),
      Codec_::default(),
      FrameDetector::new::<Preamble, { Codec_::SAMPLES_PER_PACKET }>(),
    );
    Self::new(tx, rx)
  }
}
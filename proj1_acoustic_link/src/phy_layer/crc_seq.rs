use super::common::*;
use crate::helper::CrcSeq;

#[derive(Debug)]
/// packet receive error
pub enum PacketError {
  /// the packet is lost in transmission
  Lost,
  /// the packet is received but corrupted
  Corrupt,
  /// no packet is received
  NoPacketAvaiable,
}

/// An atomic physics layer: no partial failure.  
/// packet lost/corrupt are detected
pub struct AtomicPHY {
  tx: Tx,
  tx_seq: u8,
  rx: Rx,
  rx_seq: u8,
}

type CS = CrcSeq<{ Codec_::BYTES_PER_PACKET }>;
impl AtomicPHY {
  /// number of data bytes in one packet
  pub const PACKET_BYTES: usize = CS::DATA_SIZE;
  /// number of samples in one packet
  pub const PACKET_SAMPLES: usize = Tx::SAMPLES_PER_PACKET;

  /// combine a sender and a receiver to get a physics layer object
  pub fn new(tx: Tx, rx: Rx) -> Self {
    Self {
      tx,
      tx_seq: 0,
      rx,
      rx_seq: 0,
    }
  }
}

impl PacketSender<PhyPacket, ()> for AtomicPHY {
  fn send(&mut self, mut packet: PhyPacket) -> Result<(), ()> {
    todo!()
  }
}
impl PacketReceiver<(PhyPacket, u8), PacketError> for AtomicPHY {
  /// Success: A tuple of
  /// - the received packet and
  /// - the number of lost/corrupted packets since last success.
  fn recv(&mut self) -> Result<(PhyPacket, u8), PacketError> {
    todo!()
  }

  fn recv_timeout(&mut self, timeout: std::time::Duration) -> Result<(PhyPacket, u8), PacketError> {
    let now = std::time::Instant::now();
    while now.elapsed() < timeout {
      std::thread::yield_now();
      match self.recv() {
        Ok(pack_skip) => return Ok(pack_skip),
        Err(PacketError::NoPacketAvaiable) => continue,
        Err(e) => return Err(e),
      }
    }
    Err(PacketError::NoPacketAvaiable)
  }
}

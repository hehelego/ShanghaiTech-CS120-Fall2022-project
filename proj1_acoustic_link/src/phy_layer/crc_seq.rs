use super::common::*;
use super::PlainPHY;
use crate::helper::{CrcSeq, SEQ_MOD};

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
#[derive(Default)]
pub struct AtomicPHY {
  txrx: PlainPHY,
  tx_seq: u8,
  rx_seq: u8,
}

type CS = CrcSeq<{ PlainPHY::PACKET_BYTES }>;
impl AtomicPHY {
  /// number of data bytes in one packet
  pub const PACKET_BYTES: usize = CS::DATA_SIZE;
  /// number of samples in one packet
  pub const PACKET_SAMPLES: usize = Tx::SAMPLES_PER_PACKET;

  /// combine a sender and a receiver to get a physics layer object
  pub fn new(txrx: PlainPHY) -> Self {
    Self {
      txrx,
      tx_seq: 0,
      rx_seq: 0,
    }
  }
}

impl PacketSender<PhyPacket, ()> for AtomicPHY {
  fn send(&mut self, packet: PhyPacket) -> Result<(), ()> {
    assert_eq!(packet.len(), Self::PACKET_BYTES);
    let packet = CS::add(&packet, self.tx_seq);
    self.tx_seq = (self.tx_seq + 1) % SEQ_MOD;
    self.txrx.send(packet)
  }
}
impl AtomicPHY {
  fn after_recv(&mut self, packet: PhyPacket) -> Result<(PhyPacket, u8), PacketError> {
    if let Some((packet, seq)) = CS::remove(&packet) {
      let skip = (seq - self.rx_seq + SEQ_MOD) % SEQ_MOD;
      self.rx_seq = (seq + 1) % SEQ_MOD;
      Ok((packet, skip))
    } else {
      Err(PacketError::Corrupt)
    }
  }
}
impl PacketReceiver<(PhyPacket, u8), PacketError> for AtomicPHY {
  /// Success: A tuple of
  /// - the received packet and
  /// - the number of lost/corrupted packets since last success.
  fn recv(&mut self) -> Result<(PhyPacket, u8), PacketError> {
    if let Ok(packet) = self.txrx.recv() {
      self.after_recv(packet)
    } else {
      Err(PacketError::NoPacketAvaiable)
    }
  }

  fn recv_timeout(&mut self, timeout: std::time::Duration) -> Result<(PhyPacket, u8), PacketError> {
    if let Ok(packet) = self.txrx.recv_timeout(timeout) {
      self.after_recv(packet)
    } else {
      Err(PacketError::NoPacketAvaiable)
    }
  }
}

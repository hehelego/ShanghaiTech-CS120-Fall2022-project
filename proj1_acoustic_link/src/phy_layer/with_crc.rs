use super::{PhyLayer, PlainPHY};
pub use crate::phy_packet::{Modem, PhyPacket, PreambleGen};
pub use crate::traits::{PacketReceiver, PacketSender};
use std::time::Duration;

use crc::{Crc, CRC_16_USB};

/// CRC PHY layer receive error type
#[derive(Debug)]
pub enum CrcPhyRecvErr {
  /// no packet avaiable
  NoPacket,
  /// packet received but corrupted (CRC16 checksum failed)
  Corrupt,
}

/// A PHY layer implementation with CRC16 checksum protecting each packet
/// packet corruption can be detected
#[derive(Default)]
pub struct CrcPhy(PlainPHY);

impl CrcPhy {
  /// number of bytes reserved for CRC checksum
  pub const CRC_BYTES: usize = 2;
  /// the crc16 checksum algorithm
  pub const CRC16: Crc<u16> = Crc::<u16>::new(&CRC_16_USB);

  /// combine a sender and a receiver to get a physics layer object
  pub fn new(txrx: PlainPHY) -> Self {
    Self(txrx)
  }

  fn crc_append(mut packet: PhyPacket) -> PhyPacket {
    let crc = Self::CRC16.checksum(&packet);
    let cs_low = (crc & 0x00FF) as u8;
    let cs_high = (crc & 0xFF00) as u8;
    packet.extend([cs_low, cs_high]);
    packet
  }
  fn crc_remove(packet: PhyPacket) -> Option<PhyPacket> {
    let (data, checksum) = packet.split_at(Self::PACKET_BYTES);
    let crc = Self::CRC16.checksum(data);
    let cs_low = (crc & 0x00FF) as u8;
    let cs_high = (crc & 0xFF00) as u8;
    if checksum == [cs_low, cs_high] {
      Some(PhyPacket::from(data))
    } else {
      None
    }
  }

  /// verify data integrity with crc16
  fn on_packet_arrived(&mut self, packet: PhyPacket) -> Result<PhyPacket, CrcPhyRecvErr> {
    if let Some(packet_data) = Self::crc_remove(packet) {
      Ok(packet_data)
    } else {
      println!("[CRC PHY] a packet is corrupted");
      Err(CrcPhyRecvErr::Corrupt)
    }
  }
}

impl PhyLayer for CrcPhy {
  type SendErr = ();
  type RecvErr = CrcPhyRecvErr;

  /// number of data bytes in one packet, 2 bytes used for CRC16
  const PACKET_BYTES: usize = PlainPHY::PACKET_BYTES - Self::CRC_BYTES;
  const ESTIMATED_RTT: Duration = PlainPHY::ESTIMATED_RTT;

  fn channel_free(&self) -> bool {
    self.0.channel_free()
  }
}

impl PacketSender<PhyPacket, ()> for CrcPhy {
  fn send(&mut self, packet: PhyPacket) -> Result<(), ()> {
    assert_eq!(packet.len(), Self::PACKET_BYTES);
    let packet = Self::crc_append(packet);
    self.0.send(packet)
  }
}

impl PacketReceiver<PhyPacket, CrcPhyRecvErr> for CrcPhy {
  /// Receive a packet immediately.  
  /// Success: the received packet
  /// Failed: [`RecvError`] type: no packet, packet corrupt, packet lost
  fn recv(&mut self) -> Result<PhyPacket, CrcPhyRecvErr> {
    if let Ok(packet) = self.0.recv() {
      self.on_packet_arrived(packet)
    } else {
      Err(CrcPhyRecvErr::NoPacket)
    }
  }

  /// Try to receive a packet before timeout.
  /// This is the blocking version of [`CrcPhy::recv`].  
  /// Success:the received packet
  /// Failed: [`RecvError`] type: no packet, packet corrupt, packet lost
  fn recv_timeout(&mut self, timeout: std::time::Duration) -> Result<PhyPacket, CrcPhyRecvErr> {
    if let Ok(packet) = self.0.recv_timeout(timeout) {
      self.on_packet_arrived(packet)
    } else {
      Err(CrcPhyRecvErr::NoPacket)
    }
  }

  fn recv_peek(&mut self) -> bool {
    self.0.recv_peek()
  }
}

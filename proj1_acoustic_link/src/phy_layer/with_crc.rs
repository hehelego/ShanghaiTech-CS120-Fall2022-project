use crate::{
  phy_packet::PhyPacket,
  traits::{PacketReceiver, PacketSender},
};

use super::PlainPhy;
use crc::{Algorithm, Crc, CRC_8_GSM_A};

type CrcWidth = u8;
const CRC_ALGO: Algorithm<CrcWidth> = CRC_8_GSM_A;

#[derive(Debug)]
/// the packet CRC checksum does not match the checksum section
pub struct PacketCorrupt;

/// a PHY layer with CRC16 checksum
pub struct WithCrc {
  crc: Crc<CrcWidth>,
  txrx: PlainPhy,
}

impl WithCrc {
  pub const PACKET_BYTES: usize = PlainPhy::PACKET_BYTES - 1;
  pub const PACKET_SAMPLES: usize = PlainPhy::PACKET_SAMPLES;
  fn new(crc: Crc<CrcWidth>, txrx: PlainPhy) -> Self {
    Self { crc, txrx }
  }
}
impl Default for WithCrc {
  fn default() -> Self {
    let crc = Crc::<CrcWidth>::new(&CRC_ALGO);
    let txrx = PlainPhy::default();
    Self::new(crc, txrx)
  }
}

impl PacketSender<PhyPacket, ()> for WithCrc {
  /// add crc checksum and send a packet
  fn send(&mut self, mut packet: PhyPacket) -> Result<(), ()> {
    assert_eq!(packet.len(), Self::PACKET_BYTES);
    let crc = self.crc.checksum(&packet);
    packet.push(crc);
    self.txrx.send(packet)
  }
}
impl PacketReceiver<PhyPacket, PacketCorrupt> for WithCrc {
  /// receive a packet and verify
  fn recv(&mut self) -> Result<PhyPacket, PacketCorrupt> {
    let packet = self.txrx.recv().unwrap();
    let crc_field = packet[PlainPhy::PACKET_BYTES - 1];
    let crc = self.crc.checksum(&packet);
    if crc == crc_field {
      Ok(packet[..Self::PACKET_BYTES].into())
    } else {
      Err(PacketCorrupt)
    }
  }
}

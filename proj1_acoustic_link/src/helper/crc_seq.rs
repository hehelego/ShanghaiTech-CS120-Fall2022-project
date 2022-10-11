use crc::{Crc, CRC_14_GSM};

/// a bit mask with `n` ones on the lower bits
const fn full_mask(n: usize) -> u8 {
  let n = if n <= 8 { n } else { 8 };
  const FULL: u8 = 0b_1111_1111;
  FULL >> (8 - n)
}
/// access a slice in reversed order
fn ridx(data: &[u8], idx: usize) -> &u8 {
  &data[data.len() - idx - 1]
}
/// access a slice in reversed order
fn ridx_mut(data: &mut [u8], idx: usize) -> &mut u8 {
  &mut data[data.len() - idx - 1]
}

/// the CRC-14 checksum algorithm we choose
const CRC14: Crc<u16> = Crc::<u16>::new(&CRC_14_GSM);
/// number of bits for packet sequence number
const SEQ_BITS: usize = 2;
/// number of bits for CRC checksum
const CRC_BITS: usize = 14;
/// the number of packets before the sequence number wraps around
pub const SEQ_MOD: u8 = 1 << SEQ_BITS;

/// CRC + Sequence Number helper.  
/// `PACK_SIZE`: total number bytes in one packet.
///
/// The last two bytes are preserved.
/// - byte 1: sequence number and higher 6 bits of CRC-14
/// - byte 0: lower 8 bits of CRC-14
pub struct CrcSeq<const PACK_SIZE: usize>;

impl<const PACK_SIZE: usize> CrcSeq<PACK_SIZE> {
  /// number of bytes in additional non-data section
  pub const NONDATA_SIZE: usize = 2;
  /// encoding data bytes in a packet
  pub const DATA_SIZE: usize = PACK_SIZE - Self::NONDATA_SIZE;

  /// add sequence number and checksum to a chunk of data,
  /// return a packet with crc+seq.  
  /// `data` must have exactly [`CrcSeq::DATA_SIZE`] bytes.
  pub fn pack(data: &[u8], seq: u8) -> Vec<u8> {
    assert_eq!(data.len(), Self::DATA_SIZE);
    assert!(seq < SEQ_MOD);
    let mut packet = data.to_vec();
    packet.extend([0, 0]);

    // add sequence number
    *ridx_mut(&mut packet, 1) = seq << (CRC_BITS - 8);
    // product data with CRC14 checksum
    let checksum = CRC14.checksum(&packet);
    *ridx_mut(&mut packet, 1) |= (checksum >> 8) as u8;
    *ridx_mut(&mut packet, 0) = (checksum & 0b_1111_1111) as u8;

    packet
  }

  /// try to extract the data packet and sequence number,
  /// if the data is corrupted, [`None`] is returned.
  pub fn unpack(packet: &[u8]) -> Option<(Vec<u8>, u8)> {
    assert_eq!(packet.len(), PACK_SIZE);
    let mut packet = packet.to_vec();
    // extract the sequence number field
    let seq = *ridx(&packet, 1) >> (CRC_BITS - 8);
    assert!(seq < SEQ_MOD);
    // extract the crc checksum field
    let chk = {
      let chk_high = (*ridx(&packet, 1) & full_mask(CRC_BITS - 8)) as u16;
      let chk_low = *ridx(&packet, 0) as u16;
      (chk_high << 8) | chk_low
    };
    // set the crc bits to zero
    *ridx_mut(&mut packet, 1) &= !full_mask(CRC_BITS - 8);
    *ridx_mut(&mut packet, 0) = 0;
    // check data integrity
    let verify_crc = CRC14.checksum(&packet);
    if verify_crc == chk {
      Some((packet[..Self::DATA_SIZE].to_vec(), seq))
    } else {
      None
    }
  }
}

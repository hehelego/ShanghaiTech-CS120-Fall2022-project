/// The MAC address
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Addr(pub(super) u8);
impl Addr {
  /// The number of bits for a MAC address
  pub const ADDR_BITS: usize = 4;
  /// create a MAC address from a non-negative integer
  /// **NOTE** this function should only be used by the MAC layer object
  pub const fn new(addr: u8) -> Self {
    assert!(addr < (1 << Self::ADDR_BITS));
    Self(addr)
  }
}

/// The MAC packet sequence number
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Seq(pub(super) u8);
impl Seq {
  /// The number of bits for MAC packet sequence number
  pub const SEQ_BITS: usize = 6;
  /// create a sequence number from a non-negative integer
  /// **NOTE** this function should only be used by the MAC layer object
  pub fn new(seq: u8) -> Self {
    assert!(seq < (1 << Self::SEQ_BITS));
    Self(seq)
  }
  /// get the sequence number of next packet
  /// **NOTE** this function should only be used by the MAC layer object
  pub fn next(self) -> Self {
    Self::new((self.0 + 1) & 0b_111111)
  }
}

/// The MAC packet type id
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
  Ack,
  Data,
  PingReq,
  PingReply,
}
impl Type {
  /// The number of bits for MAC packet type id
  pub const SEQ_BITS: usize = 2;
  /// MAC packet type -> type id integer
  /// **NOTE** this function should only be used in [`Packet::into_bytes`]
  pub(super) fn into_id(self) -> u8 {
    match self {
      Type::Ack => 0,
      Type::Data => 1,
      Type::PingReq => 2,
      Type::PingReply => 3,
    }
  }
  /// type id integer -> MAC packet type
  /// **NOTE** this function should only be used in [`Packet::from_bytes`]
  pub(super) fn from_id(id: u8) -> Self {
    match id {
      0 => Type::Ack,
      1 => Type::Data,
      2 => Type::PingReq,
      3 => Type::PingReply,
      _ => panic!("invalid MAC packet type id"),
    }
  }
}

/// The MAC packet, can be embeded into a PHY packet.
/// - `dest`: destination address
/// - `src`: source address
/// - `type_id`: packet type
/// - `seq`: sequence number
/// - `payload`: payload data section
///
/// **NOTE** due to constraints in rust constant generics,
/// the `payload` field length cannot be determined in compilation time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
  src: Addr,
  dest: Addr,
  type_id: Type,
  seq: Seq,
  payload: Vec<u8>,
}

const PHY_PACK_SZ: usize = crate::common::PHY_PACKET_BYTES;
use crate::common::PHY_PACKET_BYTES;
impl Packet {
  /// The number of bytes in one MAC packet
  pub const PAYLOAD_BYTES: usize = PHY_PACKET_BYTES - 2;

  /// create a new MAC packet, with the fields specified in the parameters.
  /// **NOTE** this function should only be called by the MAC layer object
  pub fn new(src: Addr, dest: Addr, type_id: Type, seq: Seq, payload: Vec<u8>) -> Self {
    assert_eq!(payload.len(), Self::PAYLOAD_BYTES);
    Self {
      src,
      dest,
      type_id,
      seq,
      payload,
    }
  }

  /// parse a MAC packet from a PHY packet
  /// **NOTE** `bytes` should have exactly [`crate::common::PHY_PACKET_BYTES`] length
  /// **NOTE** this function should only be called by the MAC layer object
  pub fn from_bytes(bytes: &[u8]) -> Self {
    assert_eq!(bytes.len(), PHY_PACK_SZ);
    let addr = bytes[0];
    let type_seq = bytes[1];
    let payload = bytes[2..].to_vec();

    let src = Addr(addr >> 4);
    let dest = Addr(addr & 0b_1111);
    let type_id = Type::from_id(type_seq >> 6);
    let seq = Seq(type_seq & 0b_111111);

    Self::new(src, dest, type_id, seq, payload)
  }
  /// build a PHY packet for a MAC packet
  /// **NOTE** the output should have exactly [`crate::common::PHY_PACKET_BYTES`] length
  /// **NOTE** this function should only be called by the MAC layer object
  pub fn into_bytes(self) -> Vec<u8> {
    let mut buf = vec![0_u8; PHY_PACK_SZ];
    buf[0] = (self.src.0 << 4) | self.dest.0;
    buf[1] = (self.type_id.into_id() << 6) | self.seq.0;
    buf[2..].copy_from_slice(&self.payload);
    buf
  }
}

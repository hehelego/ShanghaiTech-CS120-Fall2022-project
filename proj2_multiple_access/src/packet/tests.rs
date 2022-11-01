use crate::common::PHY_PACKET_BYTES;

use super::{Addr, Packet, Seq, Type};
use rand::{distributions::Standard, Rng};

const TESTS: usize = 100;

/// verify the packet then unpack identity
#[test]
fn pack_unpack() {
  for _ in 0..TESTS {
    let mut rng = rand::thread_rng();
    let src = Addr::new(rng.gen_range(0..16));
    let dest = Addr::new(rng.gen_range(0..16));
    let type_id = Type::from_id(rng.gen_range(0..4));
    let seq = Seq::new(rng.gen_range(0..64));
    let payload: Vec<u8> = rng.sample_iter(Standard).take(Packet::PAYLOAD_BYTES).collect();

    let packet = Packet::new(src, dest, type_id, seq, payload);
    let identical = Packet::from_bytes(&packet.clone().into_bytes());
    assert_eq!(packet, identical);
  }
}

/// verify the unpacket then pack identity
#[test]
fn unpack_pack() {
  for _ in 0..TESTS {
    let bytes: Vec<u8> = rand::thread_rng()
      .sample_iter(Standard)
      .take(PHY_PACKET_BYTES)
      .collect();
    let identical = Packet::from_bytes(&bytes).into_bytes();
    assert_eq!(bytes, identical);
  }
}

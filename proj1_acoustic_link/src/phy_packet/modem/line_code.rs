use crate::{
  helper::{decode_4b5b, decode_nrzi, encode_4b5b, encode_nrzi},
  phy_packet::traits::{FramePayload, Modem, PhyPacket},
  traits::{Sample, FP},
};
use bitvec::prelude::*;

/// Line Code: 4b5b + NRZI
#[derive(Default)]
pub struct LineCode;
impl LineCode {
  /// number of samples used to encode a bit
  pub const SAMPLES_PER_BIT: usize = 3;
  /// number of bits in one packet
  pub const BITS_PER_PACKET: usize = 800;
}

impl Modem for LineCode {
  const BYTES_PER_PACKET: usize = LineCode::BITS_PER_PACKET / 8;
  const SAMPLES_PER_PACKET: usize = (LineCode::SAMPLES_PER_BIT * LineCode::BITS_PER_PACKET) / 4 * 5;

  fn modulate(&mut self, bytes: &[u8]) -> FramePayload {
    assert_eq!(bytes.len(), Self::BYTES_PER_PACKET);
    let data_bits = bytes.view_bits::<Msb0>().to_owned();
    let code_bits = encode_nrzi(encode_4b5b(data_bits));
    let samples: Vec<_> = code_bits
      .into_iter()
      .flat_map(|x| {
        if x {
          [FP::ONE; Self::SAMPLES_PER_BIT]
        } else {
          [-FP::ONE; Self::SAMPLES_PER_BIT]
        }
      })
      .collect();
    samples
  }

  fn demodulate(&mut self, samples: &[FP]) -> PhyPacket {
    assert_eq!(samples.len(), Self::SAMPLES_PER_PACKET);
    let code_bits = samples
      .chunks_exact(Self::SAMPLES_PER_BIT)
      .into_iter()
      .map(|x| x.iter().fold(FP::ZERO, |acc, &x| acc + x) > FP::ZERO)
      .collect();
    let data_bits = decode_4b5b(decode_nrzi(code_bits));
    let mut bytes = vec![0; Self::BYTES_PER_PACKET];
    bytes.view_bits_mut::<Msb0>().copy_from_bitslice(&data_bits);
    bytes
  }
}
impl LineCode {
  pub fn new() -> Self {
    Self
  }
}

use crate::{
  helper::{bits_to_bytes, bytes_to_bits, dot_product},
  phy_packet::traits::{Codec, FramePayload, PhyPacket},
};

/// OFDM + PSK modulation
pub struct OFDM {}
impl OFDM {
  pub fn new() -> Self {
    Self {}
  }
}
impl Default for OFDM {
  fn default() -> Self {
    Self::new()
  }
}
// TODO: OFDM implementation
impl Codec for OFDM {
  const BYTES_PER_PACKET: usize = todo!();

  const SAMPLES_PER_PACKET: usize = todo!();

  fn encode(&mut self, bytes: &[u8]) -> FramePayload {
    todo!()
  }

  fn decode(&mut self, samples: &[f32]) -> PhyPacket {
    todo!()
  }
}

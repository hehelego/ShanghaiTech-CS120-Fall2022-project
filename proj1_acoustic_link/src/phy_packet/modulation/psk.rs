use crate::{
  helper::{bits_to_bytes, bytes_to_bits, dot_product},
  phy_packet::traits::{Codec, FramePayload, PhyPacket},
  traits::{Sample, FP},
};

/// PSK (phase shift keying)  
/// - one bit per symbol
/// - fixed frequency carrier, 0/pi phases
/// - dot product + integration for demodulation
pub struct PSK {
  symbols: [Vec<FP>; 2],
}
impl PSK {
  /// sampling rate of the digital signal
  pub const SAMPLE_RATE: usize = 48000;
  /// frequency of the carrier wave
  pub const CARRIER_FREQ: f32 = 4800.0;
  /// number of samples used to encode a bit
  pub const SAMPLES_PER_SYMBOL: usize = 40;
  /// number of bits in one packet
  pub const SYMBOLS_PER_PACKET: usize = 80;
}
impl Codec for PSK {
  const BYTES_PER_PACKET: usize = PSK::SYMBOLS_PER_PACKET / 8;

  const SAMPLES_PER_PACKET: usize = PSK::SAMPLES_PER_SYMBOL * PSK::SYMBOLS_PER_PACKET;

  fn encode(&mut self, bytes: &[u8]) -> FramePayload {
    assert_eq!(bytes.len(), Self::BYTES_PER_PACKET);

    let mut frame = FramePayload::with_capacity(Self::SAMPLES_PER_PACKET);
    bytes_to_bits(bytes)
      .into_iter()
      .for_each(|bit| frame.extend(&self.symbols[bit as usize]));
    frame
  }

  fn decode(&mut self, samples: &[FP]) -> PhyPacket {
    assert_eq!(samples.len(), Self::SAMPLES_PER_PACKET);

    let mut bits = Vec::with_capacity(Self::SYMBOLS_PER_PACKET);
    samples.chunks_exact(Self::SAMPLES_PER_SYMBOL).for_each(|symbol| {
      let sum = dot_product(symbol.iter(), self.symbols[0].iter());
      let bit = if sum < FP::ZERO { 1 } else { 0 };
      bits.push(bit);
    });
    bits_to_bytes(&bits)
  }
}
impl PSK {
  pub fn new() -> Self {
    let dt = FP::ONE / FP::from_f32(Self::SAMPLE_RATE as f32);
    let zero: Vec<_> = (0..Self::SAMPLES_PER_SYMBOL)
      .map(|i| {
        let t = dt * FP::from_f32(i as f32);
        (FP::TAU * FP::from_f32(Self::CARRIER_FREQ as f32) * t).sin()
      })
      .collect();
    let one: Vec<_> = zero.iter().map(|&x| -x).collect();
    Self { symbols: [zero, one] }
  }
}

impl Default for PSK {
  fn default() -> Self {
    Self::new()
  }
}

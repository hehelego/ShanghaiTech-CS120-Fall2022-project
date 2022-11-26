use crate::{
  helper::{bits_to_bytes, bytes_to_bits, dot_product},
  phy_packet::traits::{FramePayload, Modem, PhyPacket},
  traits::{Sample, FP},
};

fn get_bit(symbol: &[FP], reference: &[FP]) -> u8 {
  assert_eq!(symbol.len(), reference.len());
  let sum = dot_product(symbol.iter(), reference.iter());
  (sum < FP::ZERO) as _
}
fn add_vec(v_lo: &[FP], v_hi: &[FP], b_lo: u8, b_hi: u8) -> Vec<FP> {
  fn f00(x_lo: &FP, x_hi: &FP) -> FP {
    *x_lo + *x_hi
  }
  fn f10(x_lo: &FP, x_hi: &FP) -> FP {
    -*x_lo + *x_hi
  }
  fn f01(x_lo: &FP, x_hi: &FP) -> FP {
    *x_lo - *x_hi
  }
  fn f11(x_lo: &FP, x_hi: &FP) -> FP {
    -*x_lo - *x_hi
  }

  let func = match (b_lo, b_hi) {
    (0, 0) => f00,
    (0, 1) => f01,
    (1, 0) => f10,
    (1, 1) => f11,
    _ => panic!("invalid bit, not 0/1"),
  };

  v_lo.iter().zip(v_hi.iter()).map(|(x, y)| func(x, y)).collect()
}

/// 2 channel 4-PSK
pub struct PSK {
  wave_lo: Vec<FP>,
  wave_hi: Vec<FP>,
  symbols: [Vec<FP>; 4],
}
impl PSK {
  /// sampling rate of the digital signal
  pub const SAMPLE_RATE: usize = 48000;
  /// frequency of the carrier wave
  pub const CARRIER_FREQ: f32 = 8000.0;
  /// number of samples used to encode a 2-bit
  pub const SAMPLES_PER_SYMBOL: usize = 6;
  /// number of 2-bits in one packet
  pub const SYMBOLS_PER_PACKET: usize = 320;
}
impl Modem for PSK {
  const BYTES_PER_PACKET: usize = 2 * PSK::SYMBOLS_PER_PACKET / 8;

  const SAMPLES_PER_PACKET: usize = PSK::SAMPLES_PER_SYMBOL * PSK::SYMBOLS_PER_PACKET;

  fn modulate(&mut self, bytes: &[u8]) -> FramePayload {
    assert_eq!(bytes.len(), Self::BYTES_PER_PACKET);

    let mut frame = FramePayload::with_capacity(Self::SAMPLES_PER_PACKET);
    bytes_to_bits(bytes).chunks_exact(2).for_each(|bit2| {
      let (lo, hi) = (bit2[0], bit2[1]);
      let b = (hi << 1) | lo;
      frame.extend(&self.symbols[b as usize]);
    });
    frame
  }

  fn demodulate(&mut self, samples: &[FP]) -> PhyPacket {
    assert_eq!(samples.len(), Self::SAMPLES_PER_PACKET);

    let mut bits = Vec::with_capacity(Self::SYMBOLS_PER_PACKET);
    samples.chunks_exact(Self::SAMPLES_PER_SYMBOL).for_each(|symbol| {
      let b_lo = get_bit(symbol, &self.wave_lo);
      let b_hi = get_bit(symbol, &self.wave_hi);
      bits.push(b_lo);
      bits.push(b_hi);
    });
    bits_to_bytes(&bits)
  }
}
impl PSK {
  pub fn new() -> Self {
    use std::f32::consts::TAU;
    let dt = 1.0 / Self::SAMPLE_RATE as f32;
    let t = (0..Self::SAMPLES_PER_SYMBOL).map(|i| dt * i as f32);

    let f_lo = Self::CARRIER_FREQ as f32;
    let f_hi = 2.0 * Self::CARRIER_FREQ as f32;
    let wave_lo: Vec<_> = t.clone().map(|t| FP::from_f32((TAU * f_lo * t).sin())).collect();
    let wave_hi: Vec<_> = t.clone().map(|t| FP::from_f32((TAU * f_hi * t).sin())).collect();

    let symbols = [
      add_vec(&wave_lo, &wave_hi, 0, 0),
      add_vec(&wave_lo, &wave_hi, 1, 0),
      add_vec(&wave_lo, &wave_hi, 0, 1),
      add_vec(&wave_lo, &wave_hi, 1, 1),
    ];

    Self {
      wave_lo,
      wave_hi,
      symbols,
    }
  }
}

impl Default for PSK {
  fn default() -> Self {
    Self::new()
  }
}

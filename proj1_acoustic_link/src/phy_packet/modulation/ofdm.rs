use crate::{
  helper::{bits_to_bytes, bytes_to_bits, copy},
  phy_packet::traits::{Codec, FramePayload, PhyPacket},
};
use rustfft::{algorithm::Radix4, Fft, FftDirection};
type Complex = rustfft::num_complex::Complex32;

/// OFDM + PSK modulation.  
///
/// A few key points to mention:
/// - FFT for demodulation
/// - IFFT for modulation
/// - use PSK on each frequency channel
/// - add guard interval or cyclic prefix between symbols
/// - the low frequency channels are discarded as they are too noisy
pub struct OFDM {
  fft: Radix4<f32>,
  ifft: Radix4<f32>,
}
impl OFDM {
  /// number of bits in one symbol
  pub const BITS_PER_SYMBOL: usize = 4;
  /// number of samples in one symbol
  pub const SAMPLES_PER_SYMBOL: usize = Self::N + Self::M;
  /// number of bits in one packet
  pub const SYMBOLS_PER_PACKET: usize = 18;

  /// size of FFT/IFFT
  pub const N: usize = 64;
  /// samples in the cyclic prefix
  pub const M: usize = 16;
  /// first encoding frequency point
  pub const START: usize = 5;

  // for fft scaling
  const UNIT: f32 = 1.0 / 4.0;
  const ZERO: Complex = Complex::new(Self::UNIT, 0.0);
  const ONE: Complex = Complex::new(-Self::UNIT, 0.0);
  const PHASE: [Complex; 2] = [Self::ZERO, Self::ONE];

  pub fn new() -> Self {
    Self {
      fft: Radix4::new(Self::N, FftDirection::Forward),
      ifft: Radix4::new(Self::N, FftDirection::Inverse),
    }
  }

  fn encode_symbol(&self, buf: &mut [Complex], symbol: &mut [f32], bits: &[u8]) {
    buf.iter_mut().for_each(|x| *x = Complex::default());
    let (cp, symbol) = symbol.split_at_mut(Self::M);

    for (i, bit) in bits.iter().enumerate() {
      let val = Self::PHASE[*bit as usize];
      let j = Self::START + i;
      buf[j] = val;
    }
    self.ifft.process(buf);

    copy(cp.iter_mut(), buf[Self::N - Self::M..].iter().map(|x| x.re));
    copy(symbol.iter_mut(), buf.iter().map(|x| x.re));
  }
  fn decode_symbol(&self, buf: &mut [Complex], symbol: &[f32], bits: &mut [u8]) {
    buf.iter_mut().for_each(|x| *x = Complex::default());
    let (_cp, symbol) = symbol.split_at(Self::M);

    copy(buf.iter_mut(), symbol.iter().map(|x| Complex::new(*x, 0.0)));
    self.fft.process(buf);

    for (i, bit) in bits.iter_mut().enumerate() {
      let j = Self::START + i;
      let val = buf[j].re;
      *bit = if val > 0.0 { 0 } else { 1 };
    }
  }
}

impl Default for OFDM {
  fn default() -> Self {
    Self::new()
  }
}

impl Codec for OFDM {
  const BYTES_PER_PACKET: usize = OFDM::SYMBOLS_PER_PACKET * OFDM::BITS_PER_SYMBOL / 8;
  const SAMPLES_PER_PACKET: usize = OFDM::SYMBOLS_PER_PACKET * OFDM::SAMPLES_PER_SYMBOL;

  fn encode(&mut self, bytes: &[u8]) -> FramePayload {
    assert_eq!(bytes.len(), Self::BYTES_PER_PACKET);

    let mut frame = vec![0.0; Self::SAMPLES_PER_PACKET];
    let mut buf = [Complex::default(); Self::N];
    frame
      .chunks_exact_mut(Self::SAMPLES_PER_SYMBOL)
      .zip(bytes_to_bits(bytes).chunks_exact(Self::BITS_PER_SYMBOL))
      .for_each(|(symbol, bits)| self.encode_symbol(&mut buf, symbol, bits));
    frame
  }

  fn decode(&mut self, samples: &[f32]) -> PhyPacket {
    assert_eq!(samples.len(), Self::SAMPLES_PER_PACKET);

    let mut bits = [0; OFDM::SYMBOLS_PER_PACKET * OFDM::BITS_PER_SYMBOL];
    let mut buf = [Complex::default(); Self::N];
    samples
      .chunks_exact(Self::SAMPLES_PER_SYMBOL)
      .zip(bits.chunks_exact_mut(Self::BITS_PER_SYMBOL))
      .for_each(|(symbol, bits)| self.decode_symbol(&mut buf, symbol, bits));
    bits_to_bytes(&bits)
  }
}

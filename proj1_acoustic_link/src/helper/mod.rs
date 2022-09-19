/// Generate a [chirp](https://en.wikipedia.org/wiki/Chirp) digital signal.  
/// The instantaneous frequency change linearly from `freq_a` to `freq_b`.  
/// The signal contains exactly `len` samples,
/// where the sampling rate is `sample_rate` samples per second.
pub fn chirp(freq_a: f32, freq_b: f32, len: usize, sample_rate: usize) -> impl ExactSizeIterator<Item = f32> {
  let dt = 1.0 / sample_rate as f32;
  let duration = dt * len as f32;
  let df_dt = (freq_b - freq_a) / duration;

  use std::f32::consts::{PI, TAU};
  // delta phase / delta time = 2*pi*freq_a + 2*pi*df_dt*t
  (0..len).map(move |i| {
    let t = i as f32 * dt;
    let phase = TAU * freq_a * t + PI * df_dt * t * t;
    phase.sin()
  })
}

/// Compute the dot product of two sequences.  
/// Panic if the two given sequences have unequal lengths.
pub fn dot_product<'a, 'b, Ia, Ib>(seq_a: Ia, seq_b: Ib) -> f32
where
  Ia: ExactSizeIterator<Item = &'a f32>,
  Ib: ExactSizeIterator<Item = &'a f32>,
{
  assert_eq!(seq_a.len(), seq_b.len());
  seq_a.zip(seq_b).fold(0.0, |sum, (x, y)| sum + x * y)
}

/// Copy samples from `src` to fill `dest`.  
/// Return the number of copied samples.
pub fn copy<'a, T, D, S>(dest: D, src: S) -> usize
where
  T: 'a + Clone,
  D: Iterator<Item = &'a mut T>,
  S: Iterator<Item = T>,
{
  dest.zip(src).fold(0, |n, (x, y)| {
    *x = y;
    n + 1
  })
}

/// bytes to bits. the bits order does not matter.
pub fn bytes_to_bits(bytes: &[u8]) -> Vec<u8> {
  let mut bits = Vec::with_capacity(bytes.len() * 8);
  bytes
    .into_iter()
    .for_each(|byte| (0..8).for_each(|i| bits.push((byte >> i) & 1)));
  bits
}
/// the reverse process of [`bytes_to_bits`].
pub fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
  assert_eq!(bits.len() % 8, 0);
  let mut bytes = Vec::with_capacity(bits.len() / 8);
  bits
    .chunks_exact(8)
    .for_each(|bits| bytes.push(bits.iter().rev().fold(0, |s, bit| (s << 1) | bit)));
  bytes
}

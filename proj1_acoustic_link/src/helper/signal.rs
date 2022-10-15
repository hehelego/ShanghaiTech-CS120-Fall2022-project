use crate::traits::{Sample, FP};

/// Generate a [chirp](https://en.wikipedia.org/wiki/Chirp) digital signal.  
/// The instantaneous frequency change linearly from `freq_a` to `freq_b`.  
/// The signal contains exactly `len` samples,
/// where the sampling rate is `sample_rate` samples per second.
pub fn chirp(freq_a: FP, freq_b: FP, len: usize, sample_rate: usize) -> impl ExactSizeIterator<Item = FP> {
  let dt = FP::ONE / FP::from_f32(sample_rate as f32);
  let duration = dt * FP::from_f32(len as f32);
  let df_dt = (freq_b - freq_a) / duration;

  // delta phase / delta time = 2*pi*freq_a + 2*pi*df_dt*t
  (0..len).map(move |i| {
    let t = FP::from_f32(i as f32) * dt;
    let phase = FP::TAU * freq_a * t + FP::PI * df_dt * t * t;
    phase.sin()
  })
}

/// Compute the dot product of two sequences.  
/// Panic if the two given sequences have unequal lengths.
pub fn dot_product<'a, 'b, Ia, Ib>(seq_a: Ia, seq_b: Ib) -> FP
where
  Ia: ExactSizeIterator<Item = &'a FP>,
  Ib: ExactSizeIterator<Item = &'a FP>,
{
  assert_eq!(seq_a.len(), seq_b.len());
  seq_a.zip(seq_b).fold(FP::ZERO, |sum, (x, y)| sum + x * y)
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

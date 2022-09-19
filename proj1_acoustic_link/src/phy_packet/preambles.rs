use crate::helper::chirp;

use super::PreambleGenerator;

use std::iter;

type ITER = std::vec::IntoIter<f32>;

/// an empty preamble sequence
pub struct Empty;
impl PreambleGenerator for Empty {
  type PreambleSequence = iter::Empty<f32>;

  const PREAMBLE_LEN: usize = 0;

  fn generate_preamble() -> Self::PreambleSequence {
    iter::empty()
  }
}

/// an chirp signal preamble sequence, the frequency goes up then down.  
/// **NOTE** Due to lack of rustc features, the following parameters are not configurable.  
pub struct ChirpUpDown;

impl ChirpUpDown {
  /// the lowest frequency  
  pub const FA: f32 = 3000.0;
  /// the highest frequency  
  pub const FB: f32 = 6000.0;
  /// number of samples  
  pub const N: usize = 440;
  /// the sampling frequency  
  pub const FS: usize = 48000;
}

impl PreambleGenerator for ChirpUpDown {
  type PreambleSequence = ITER;

  const PREAMBLE_LEN: usize = ChirpUpDown::N;

  fn generate_preamble() -> Self::PreambleSequence {
    let fa = ChirpUpDown::FA;
    let fb = ChirpUpDown::FB;
    let m = ChirpUpDown::N / 2;
    let fs = ChirpUpDown::FS;

    let data: Vec<_> = chirp(fa, fb, m, fs).chain(chirp(fb, fa, m, fs)).collect();
    data.into_iter()
  }
}

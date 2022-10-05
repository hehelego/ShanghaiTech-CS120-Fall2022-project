use crate::helper::chirp;

use super::PreambleGen;

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

impl PreambleGen for ChirpUpDown {
  const PREAMBLE_LEN: usize = ChirpUpDown::N;

  fn generate() -> Vec<f32> {
    let fa = ChirpUpDown::FA;
    let fb = ChirpUpDown::FB;
    let m = ChirpUpDown::N / 2;
    let fs = ChirpUpDown::FS;

    chirp(fa, fb, m, fs).chain(chirp(fb, fa, m, fs)).collect()
  }
}

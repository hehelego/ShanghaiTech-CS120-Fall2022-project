use crate::helper::chirp;
use crate::traits::{Sample, FP};

use super::{traits::FramePreamble, PreambleGen};

/// an chirp signal preamble sequence, the frequency goes up then down.  
/// **NOTE** Due to lack of rustc features, the following parameters are not configurable.  
pub struct ChirpUpDown {
  samples: Vec<FP>,
  norm: FP,
}

impl ChirpUpDown {
  /// the lowest frequency  
  pub const FA: f32 = 3000.0;
  /// the highest frequency  
  pub const FB: f32 = 6000.0;
  /// number of samples  
  pub const N: usize = 440;
  /// the sampling frequency  
  pub const FS: usize = 48000;

  pub fn new() -> ChirpUpDown {
    let fa = FP::from_f32(ChirpUpDown::FA);
    let fb = FP::from_f32(ChirpUpDown::FB);
    let m = ChirpUpDown::N / 2;
    let fs = ChirpUpDown::FS;

    let samples: Vec<FP> = chirp(fa, fb, m, fs).chain(chirp(fb, fa, m, fs)).collect();
    let norm = samples.iter().fold(FP::ZERO, |s, &x| s + x * x).sqrt();
    Self { samples, norm }
  }
}

impl Default for ChirpUpDown {
  fn default() -> Self {
    Self::new()
  }
}

impl PreambleGen for ChirpUpDown {
  const PREAMBLE_LEN: usize = ChirpUpDown::N;

  fn samples(&self) -> FramePreamble {
    self.samples.clone()
  }
  fn iter(&self) -> std::slice::Iter<FP> {
    self.samples.iter()
  }
  fn len(&self) -> usize {
    self.samples.len()
  }
  fn norm(&self) -> FP {
    self.norm
  }

  fn generate() -> Self {
    Self::new()
  }
}

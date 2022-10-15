use proj1_acoustic_link::traits::{Sample, FP};
use std::time::Duration;

use proj1_acoustic_link::{sample_stream::CpalOutStream, traits::OutStream, DefaultConfig};

#[test]
#[ignore]
fn part2_ck1() {
  // Create the samples; OH, MY RUSTNESS, lame compile time calculation.
  const TOTAL_SECS: usize = 10;
  const TOTAL_SAMPLES: usize = DefaultConfig::SAMPLE_RATE as usize * TOTAL_SECS;
  let samples: Vec<FP> = (1..TOTAL_SAMPLES)
    .map(|steps| {
      let ptime: FP = FP::from_f32(steps as f32) / FP::from_f32(DefaultConfig::SAMPLE_RATE as f32);
      (FP::TAU * FP::from_f32(1000.0) * ptime).sin() + (FP::TAU * FP::from_f32(10000.0) * ptime).sin()
    })
    .collect();
  // Play the sound
  let mut cpal_out_stream = CpalOutStream::default();
  cpal_out_stream.write(samples.as_slice()).unwrap();
  std::thread::sleep(Duration::from_secs(10));
}

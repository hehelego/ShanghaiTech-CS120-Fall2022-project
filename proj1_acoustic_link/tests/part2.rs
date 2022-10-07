use std::time::Duration;

use proj1_acoustic_link::{sample_stream::CpalOutStream, traits::OutStream, DefaultConfig};

#[test]
#[ignore]
fn part2_ck1() {
  // Create the samples; OH, MY RUSTNESS, lame compile time calculation.
  const TOTAL_SECS: usize = 10;
  const TOTAL_SAMPLES: usize = DefaultConfig::SAMPLE_RATE as usize * TOTAL_SECS;
  let samples: Vec<f32> = (1..TOTAL_SAMPLES)
    .map(|steps| {
      let ptime: f32 = (steps as f32) / (DefaultConfig::SAMPLE_RATE as f32);
      (std::f32::consts::TAU * 1000.0 * ptime).sin() + (std::f32::consts::TAU * 10000.0 * ptime).sin()
    })
    .collect();
  // Play the sound
  let mut cpal_out_stream = CpalOutStream::default();
  cpal_out_stream.write(samples.as_slice()).unwrap();
  std::thread::sleep(Duration::from_secs(10));
}

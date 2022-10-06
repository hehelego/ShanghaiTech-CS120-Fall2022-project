use std::time::Duration;

use proj1_acoustic_link::{
  sample_stream::{CpalInStream, CpalOutStream, HoundInStream},
  traits::{InStream, OutStream},
  DefaultConfig,
};

#[test]
#[ignore]
pub fn part1_ck1() {
  // Buffer size
  const BUFSIZE: usize = 1024;
  // record duration
  const RECDURATION: u64 = 10;
  // buffer transfer data between in stream and out stream.
  let mut buf = [0.0; BUFSIZE];
  // Create cpal_in_stream and record for RECDURATION.
  let mut cpal_in_stream = CpalInStream::default();
  std::thread::sleep(std::time::Duration::from_secs(RECDURATION));
  cpal_in_stream.pause();
  // Play the result.
  let mut cpal_out_stream = CpalOutStream::default();
  while cpal_in_stream.read(&mut buf).unwrap() != 0 {
    cpal_out_stream.write(&buf).unwrap();
  }
  std::thread::sleep(std::time::Duration::from_secs(RECDURATION));
}

#[test]
#[ignore]
pub fn part1_ck2() {
  const RECDURATION: u64 = 10;
  const FILENAME: &str = "winter.wav";
  let mut buf = [0.0; DefaultConfig::BUFFER_SIZE];
  let mut hound_in_stream = HoundInStream::open(FILENAME);
  let mut cpal_out_stream = CpalOutStream::default();
  let mut cpal_in_stream = CpalInStream::default();
  while hound_in_stream.read(&mut buf).unwrap() != 0 {
    cpal_out_stream.write(&buf).unwrap();
  }
  std::thread::sleep(Duration::from_secs(RECDURATION));
  cpal_out_stream.clear();
  cpal_in_stream.pause();
  while cpal_in_stream.read(&mut buf).unwrap() != 0 {
    cpal_out_stream.write(&mut buf).unwrap();
  }
  std::thread::sleep(Duration::from_secs(RECDURATION));
}

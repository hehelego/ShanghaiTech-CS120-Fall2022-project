use crate::{
  sample_stream::{CpalInStream, CpalOutStream},
  traits::{InStream, OutStream},
};

#[test]
pub fn part1_ck1() {
  // Buffer size
  const BUFSIZE: usize = 1024;
  // record duration
  const RECDURATION: u64 = 10;
  // buffer transfer data between in stream and out stream.
  let mut buf: [f32; BUFSIZE] = [0.0; BUFSIZE];
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

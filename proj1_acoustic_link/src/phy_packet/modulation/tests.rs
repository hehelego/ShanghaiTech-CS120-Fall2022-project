use rand::{
  distributions::{Distribution, Standard},
  Rng,
};

use super::{OFDM, PSK};
use crate::phy_packet::Codec;

const CODEC_TESTS: usize = 1000;

/// encode/decode identity in ideal transmission channel
fn test_ideal<T: Codec>(mut codec: T) {
  let bytes: Vec<u8> = rand::thread_rng()
    .sample_iter(Standard)
    .take(T::BYTES_PER_PACKET)
    .collect();

  let encoded = codec.encode(&bytes);
  let decoded = codec.decode(&encoded);
  assert_eq!(bytes.as_slice(), decoded.as_slice());
}
/// encode/decode identity in noisy channel, where the noise is distributed as `noise_dist`.
fn test_noisy<T: Codec, D: Distribution<f32>>(mut codec: T, noise_dist: D) {
  let bytes: Vec<u8> = rand::thread_rng()
    .sample_iter(Standard)
    .take(T::BYTES_PER_PACKET)
    .collect();

  let encoded = codec.encode(&bytes);
  let received: Vec<_> = encoded
    .into_iter()
    .zip(rand::thread_rng().sample_iter(noise_dist))
    .map(|(x, y)| x + y)
    .collect();
  let decoded = codec.decode(&received);
  assert_eq!(bytes.as_slice(), decoded.as_slice());
}

/// PSK decode in an ideal channel
#[test]
fn psk_ideal() {
  for _ in 0..CODEC_TESTS {
    test_ideal(PSK::new());
  }
}
/// PSK decode in noisy channel, where the noise is distributed as Uniform(-1,+1).
#[test]
fn psk_noise() {
  for _ in 0..CODEC_TESTS {
    test_noisy(PSK::new(), Standard);
  }
}

/// OFDM decode in an ideal channel
#[test]
fn ofdm_ideal() {
  for _ in 0..CODEC_TESTS {
    test_ideal(OFDM::new());
  }
}
/// OFDM decode in noisy channel, where the noise is distributed as Uniform(-1,+1).
#[test]
fn ofdm_noise() {
  for _ in 0..CODEC_TESTS {
    test_noisy(OFDM::new(), Standard);
  }
}

/// OFDM encode/decode + ideal transmission through WAV file
#[test]
fn ofdm_wav_once() {
  use crate::sample_stream::{HoundInStream, HoundOutStream};
  use crate::traits::{InStream, OutStream};

  let mut codec = OFDM::new();
  let bytes: Vec<u8> = rand::thread_rng()
    .sample_iter(Standard)
    .take(OFDM::BYTES_PER_PACKET * CODEC_TESTS)
    .collect();

  let mut out_stream = HoundOutStream::create("ofdm_test.wav");
  bytes
    .chunks_exact(OFDM::BYTES_PER_PACKET)
    .for_each(|pack| out_stream.write_exact(&codec.encode(pack)).unwrap());
  out_stream.finalize();

  let mut in_stream = HoundInStream::open("ofdm_test.wav");
  let mut received = vec![0.0; OFDM::SAMPLES_PER_PACKET * CODEC_TESTS];
  in_stream.read_exact(&mut received).unwrap();
  let decoded: Vec<_> = received
    .chunks_exact(OFDM::SAMPLES_PER_PACKET)
    .flat_map(|pack| codec.decode(pack))
    .collect();
  assert_eq!(bytes.as_slice(), decoded.as_slice());
}

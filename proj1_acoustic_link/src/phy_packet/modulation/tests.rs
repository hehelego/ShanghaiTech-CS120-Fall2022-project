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

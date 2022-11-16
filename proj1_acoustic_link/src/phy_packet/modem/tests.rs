use rand::{
  distributions::{Distribution, Standard},
  Rng,
};

use crate::phy_packet::Modem;
use crate::traits::{Sample, FP};

const MODEM_TESTS: usize = 1000;

/// encode/decode identity in ideal transmission channel
fn test_ideal<T: Modem>(mut modem: T) {
  let bytes: Vec<u8> = rand::thread_rng()
    .sample_iter(Standard)
    .take(T::BYTES_PER_PACKET)
    .collect();

  let encoded = modem.modulate(&bytes);
  let decoded = modem.demodulate(&encoded);
  assert_eq!(bytes.as_slice(), decoded.as_slice());
}
/// encode/decode identity in noisy channel, where the noise is distributed as `noise_dist`.
fn test_noisy<T: Modem, D: Distribution<f32>>(mut modem: T, noise_dist: D) {
  let bytes: Vec<u8> = rand::thread_rng()
    .sample_iter(Standard)
    .take(T::BYTES_PER_PACKET)
    .collect();

  let encoded = modem.modulate(&bytes);
  let received: Vec<_> = encoded
    .into_iter()
    .zip(rand::thread_rng().sample_iter(noise_dist).map(FP::from_f32))
    .map(|(x, y)| x + y)
    .collect();
  let decoded = modem.demodulate(&received);
  assert_eq!(bytes.as_slice(), decoded.as_slice());
}

/// PSK decode in an ideal channel
#[test]
fn psk_ideal() {
  for _ in 0..MODEM_TESTS {
    test_ideal(crate::phy_packet::modem::psk::PSK::new());
  }
}
/// PSK decode in noisy channel, where the noise is distributed as Uniform(-1,+1).
#[test]
fn psk_noise() {
  for _ in 0..MODEM_TESTS {
    test_noisy(crate::phy_packet::modem::psk::PSK::new(), Standard);
  }
}

/// mutli-PSK decode in an ideal channel
#[test]
fn multipsk_ideal() {
  for _ in 0..MODEM_TESTS {
    test_ideal(crate::phy_packet::modem::proj2_modem::PSK::new());
  }
}
/// multi-PSK decode in noisy channel, where the noise is distributed as Uniform(-1,+1).
#[test]
fn multipsk_noise() {
  for _ in 0..MODEM_TESTS {
    test_noisy(crate::phy_packet::modem::proj2_modem::PSK::new(), Standard);
  }
}

/// OFDM decode in an ideal channel
#[test]
#[cfg(not(feature = "nofloat"))]
fn ofdm_ideal() {
  for _ in 0..MODEM_TESTS {
    test_ideal(super::OFDM::new());
  }
}
/// OFDM decode in noisy channel, where the noise is distributed as Uniform(-1,+1).
#[test]
#[cfg(not(feature = "nofloat"))]
fn ofdm_noise() {
  for _ in 0..MODEM_TESTS {
    test_noisy(super::OFDM::new(), Standard);
  }
}

/// OFDM encode/decode + ideal transmission through WAV file
#[test]
#[cfg(not(feature = "nofloat"))]
fn ofdm_wav_once() {
  use super::OFDM;
  use crate::sample_stream::{HoundInStream, HoundOutStream};
  use crate::traits::{InStream, OutStream};

  let mut modem = OFDM::new();
  let bytes: Vec<u8> = rand::thread_rng()
    .sample_iter(Standard)
    .take(OFDM::BYTES_PER_PACKET * MODEM_TESTS)
    .collect();

  let mut out_stream = HoundOutStream::create("ofdm_test.wav");
  bytes
    .chunks_exact(OFDM::BYTES_PER_PACKET)
    .for_each(|pack| out_stream.write_exact(&modem.modulate(pack)).unwrap());
  out_stream.finalize();

  let mut in_stream = HoundInStream::open("ofdm_test.wav");
  let mut received = vec![FP::ZERO; OFDM::SAMPLES_PER_PACKET * MODEM_TESTS];
  in_stream.read_exact(&mut received).unwrap();
  let decoded: Vec<_> = received
    .chunks_exact(OFDM::SAMPLES_PER_PACKET)
    .flat_map(|pack| modem.demodulate(pack))
    .collect();
  assert_eq!(bytes.as_slice(), decoded.as_slice());
}

/// line code decode in an ideal channel
#[test]
fn lc_ideal() {
  for _ in 0..MODEM_TESTS {
    test_ideal(super::LineCode::new());
  }
}
/// line code decode in noisy channel, where the noise is distributed as Uniform(-1,+1).
#[test]
fn lc_noise() {
  for _ in 0..MODEM_TESTS {
    test_noisy(super::LineCode::new(), Standard);
  }
}

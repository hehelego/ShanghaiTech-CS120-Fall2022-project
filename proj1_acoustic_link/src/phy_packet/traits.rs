use crate::traits::FP;
/// PHY layers send/receive packets of type [`PhyPacket`], which is a fixed size bytes slice
pub type PhyPacket = Vec<u8>;

/// the sequence of PCM samples put at the begining of each [`PhyPacket`] in acoustic channel.
pub type FramePreamble = Vec<FP>;
/// the sequence of PCM samples used to encode bytes of a [`PhyPacket`] in acoustic channel.
pub type FramePayload = Vec<FP>;

/// types that can generate preamble sequence
pub trait PreambleGen {
  /// number of samples in the preamble sequence
  const PREAMBLE_LEN: usize;

  /// generate the preamble samples, should contain exactly [`Self::PREAMBLE_LEN`] samples.
  fn samples(&self) -> FramePreamble;
  fn norm(&self) -> FP;
  fn len(&self) -> usize;
  fn generate() -> Self;
  fn iter(&self) -> std::slice::Iter<FP>;
}

/// type traits for encoding/decoding [`PhyPacket`]
pub trait Modem: Default {
  /// number of bytes in one packet
  const BYTES_PER_PACKET: usize;
  /// number of samples in one packet
  const SAMPLES_PER_PACKET: usize;

  /// Encode a chunk of bytes into a sequence of PCM samples.  
  /// The given data should have exactly [`Self::BYTES_PER_PACKET`] bytes.
  /// The returned sequence should have exactly [`Self::SAMPLES_PER_PACKET`] samples.
  fn modulate(&mut self, bytes: &[u8]) -> FramePayload;
  /// Decode a chunk of bytes from a sequence of PCM samples.  
  /// The given sequence should have exactly [`Self::SAMPLES_PER_PACKET`] samples.
  /// The return data should have exactly [`Self::BYTES_PER_PACKET`] bytes.
  fn demodulate(&mut self, samples: &[FP]) -> PhyPacket;
}

/// type traits for frame detector strategy
pub trait FrameDetector {
  /// Update the detector state when a new sample is received.  
  /// Return the a frame payload section if we detect any frame.
  fn on_sample(&mut self, sample: FP) -> Option<FramePayload>;
}

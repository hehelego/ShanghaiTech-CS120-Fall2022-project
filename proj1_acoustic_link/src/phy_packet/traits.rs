use crate::traits::{PacketReceiver, PacketSender};

/// packet sender types that can send [`PhyPacket`]
pub trait PhyPacketSender<E>: PacketSender<PhyPacket, E> {}
impl<PS, E> PhyPacketSender<E> for PS where PS: PacketSender<PhyPacket, E> {}

/// packet receiver types that can receive [`PhyPacket`]
pub trait PhyPacketReceiver<E>: PacketReceiver<PhyPacket, E> {}
impl<PR, E> PhyPacketReceiver<E> for PR where PR: PacketReceiver<PhyPacket, E> {}

/// A [`PhyPacket`] is a packet of data in the physics layer
pub type PhyPacket = Vec<u8>;

/// A [`Frame`] is a sequence of PCM samples, representing a [`PhyPacket`]
pub type Frame = Vec<f32>;

/// types that can generate preamble sequence
pub trait PreambleGenerator {
  /// type of the generated preamble sequence
  type PreambleSequence: ExactSizeIterator<Item = f32>;

  /// number of samples in the preamble sequence
  const PREAMBLE_LEN: usize;

  /// generate the preamble samples, should contain exactly [`Self::PREAMBLE_LEN`] samples.
  fn generate_preamble() -> Self::PreambleSequence;
}

/// type traits for encoding/decoding [`PhyPacket`]
pub trait Codec {
  /// number of bytes in one packet
  const BYTES_PER_PACKET: usize;
  /// number of samples in one packet
  const SAMPLES_PER_PACKET: usize;

  /// Encode a chunk of bytes into a sequence of PCM samples.  
  /// The given data should have exactly [`Self::BYTES_PER_PACKET`] bytes.
  /// The returned sequence should have exactly [`Self::SAMPLES_PER_PACKET`] samples.
  fn encode(&mut self, bytes: &[u8]) -> Frame;
  /// Decode a chunk of bytes from a sequence of PCM samples.  
  /// The given sequence should have exactly [`Self::SAMPLES_PER_PACKET`] samples.
  /// The return data should have exactly [`Self::BYTES_PER_PACKET`] bytes.
  fn decode(&mut self, samples: &[f32]) -> PhyPacket;

  /// create a codec
  fn new() -> Self;
}

/// type traits for frame detector strategy
pub trait FrameDetector {
  /// Update the detector state when a new sample is received.  
  /// Return a frame of samples if we detect any.
  fn update(&mut self, sample: f32) -> Option<Vec<f32>>;

  /// Create a frame detector.  
  /// To create a frame detector,
  /// the preamble sequence to detect and
  /// the length of the payload section of a frame
  /// should be given.
  fn new(preamble_samples: Vec<f32>, payload_samples: usize) -> Self;
}

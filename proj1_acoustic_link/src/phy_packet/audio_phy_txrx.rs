use super::{traits::PhyPacket, Codec, FrameDetector, PreambleGenerator};
use crate::{
  sample_stream::{SampleInStream, SampleOutStream},
  traits::PacketSender,
};
use std::marker::PhantomData;

/// A send only PHY layer object.  
/// - PG: preamble generator
/// - CC: modulation encoder/decoder
/// - FD: frame detector
/// - SS: sample input stream
/// - E: sample input stream error type
pub struct PhySender<PG, CC, FD, SS, E> {
  _pg: PhantomData<PG>,
  _err: PhantomData<E>,
  preamble_samples: Vec<f32>,
  codec: CC,
  frame_detector: FD,
  sample_stream: SS,
}

impl<PG, CC, FD, SS, E> PhySender<PG, CC, FD, SS, E>
where
  PG: PreambleGenerator,
  CC: Codec,
  FD: FrameDetector,
  SS: SampleOutStream<E>,
{
  pub fn new(stream_in: SS) -> Self {
    let preamble_samples: Vec<_> = PG::generate_preamble().collect();
    let codec = CC::new();
    let frame_detector = FD::new(preamble_samples.clone(), CC::SAMPLES_PER_PACKET);
    let sample_stream = stream_in;

    Self {
      _pg: PhantomData::default(),
      _err: PhantomData::default(),
      preamble_samples,
      codec,
      frame_detector,
      sample_stream,
    }
  }

  pub const SAMPLES_PER_PACKET: usize = PG::PREAMBLE_LEN + CC::SAMPLES_PER_PACKET;
}
impl<PG, CC, FD, SS, E> PacketSender<PhyPacket, E> for PhySender<PG, CC, FD, SS, E>
where
  PG: PreambleGenerator,
  CC: Codec,
  FD: FrameDetector,
  SS: SampleOutStream<E>,
{
  fn send(&mut self, packet: PhyPacket) -> Result<(), E> {
    todo!()
  }
}

// TODO: PHY layer receiver

// TODO: PHY layer sender&receiver

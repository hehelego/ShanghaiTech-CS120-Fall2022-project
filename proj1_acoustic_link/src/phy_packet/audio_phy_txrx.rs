use super::{traits::PhyPacket, Codec, Frame, FrameDetector, PreambleGenerator};
use crate::{
  sample_stream::{SampleInStream, SampleOutStream},
  traits::{PacketReceiver, PacketSender},
};
use std::{
  marker::PhantomData,
  sync::mpsc::{Receiver, Sender},
  thread::JoinHandle,
};

/// A send only PHY layer object.  
/// - PG: preamble generator
/// - CC: modulation encoder/decoder
/// - SS: sample input stream
/// - E: sample input stream error type
pub struct PhySender<PG, CC, SS, E> {
  _pg: PhantomData<PG>,
  _err: PhantomData<E>,
  preamble_samples: Vec<f32>,
  codec: CC,
  sample_stream: SS,
}

impl<PG, CC, SS, E> PhySender<PG, CC, SS, E>
where
  PG: PreambleGenerator,
  CC: Codec,
  SS: SampleOutStream<E>,
{
  pub fn new(stream_in: SS) -> Self {
    let preamble_samples: Vec<_> = PG::generate_preamble().collect();
    let codec = CC::new();
    let sample_stream = stream_in;

    Self {
      _pg: PhantomData::default(),
      _err: PhantomData::default(),
      preamble_samples,
      codec,
      sample_stream,
    }
  }

  pub const SAMPLES_PER_PACKET: usize = PG::PREAMBLE_LEN + CC::SAMPLES_PER_PACKET;
}
impl<PG, CC, SS, E> PacketSender<PhyPacket, E> for PhySender<PG, CC, SS, E>
where
  PG: PreambleGenerator,
  CC: Codec,
  SS: SampleOutStream<E>,
{
  /// frame = warm up + preamble + payload  
  /// - warm up: random samples whose absolute value is cloes to 1.0
  /// - preamble: predefined samples
  /// - payload: output of modulation on packet bytes
  /// NOTE: write them to the underlying stream together with `write_once`
  fn send(&mut self, packet: PhyPacket) -> Result<(), E> {
    todo!()
  }
}

/// A receive only PHY layer object.  
/// - PG: preamble generator
/// - CC: modulation encoder/decoder
/// - FD: frame detector
/// - SS: sample output stream
/// - E: sample output stream error type
pub struct PhyReceiver<PG, CC, FD, SS, E> {
  _pg: PhantomData<PG>,
  _fd: PhantomData<FD>,
  _ss: PhantomData<SS>,
  _err: PhantomData<E>,
  codec: CC,
  frame_rx: Receiver<Frame>,
  exit_tx: Sender<()>,
  handler: Option<JoinHandle<()>>,
}

impl<PG, CC, FD, SS, E> PhyReceiver<PG, CC, FD, SS, E>
where
  PG: PreambleGenerator,
  CC: Codec,
  FD: FrameDetector,
  SS: SampleInStream<E>,
{
  /// A separated worker thread repeatedly do the procedure
  /// 0. exit if notified by exit channel
  /// 1. fetch samples from underlying stream
  /// 2. push them to frame detector
  /// 3. if a frame is detected, send it to the PhyReceiver through a channel
  fn worker() {
    todo!();
  }

  pub fn new(stream_in: SS) -> Self {
    // create worker, transfer data
    todo!()
  }
}

impl<PG, CC, FD, SS, E> PacketReceiver<PhyPacket, E> for PhyReceiver<PG, CC, FD, SS, E>
where
  PG: PreambleGenerator,
  CC: Codec,
  FD: FrameDetector,
  SS: SampleInStream<E>,
{
  // receive frame from the channel and then demodulate the signal
  fn recv(&mut self) -> Result<PhyPacket, E> {
    todo!()
  }
}

impl<PG, CC, FD, SS, E> Drop for PhyReceiver<PG, CC, FD, SS, E> {
  // notify the worker thread to exit
  // wait for the worker thread to stop
  fn drop(&mut self) {
    self.exit_tx.send(()).unwrap();
    if let Some(worker) = self.handler.take() {
      worker.join().unwrap();
    }
  }
}

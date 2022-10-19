use super::{traits::PhyPacket, FrameDetector, FramePayload, Modem, PreambleGen};
use crate::{
  traits::{InStream, OutStream, PacketReceiver, PacketSender, Sample, FP},
  DefaultConfig,
};
use std::{
  marker::PhantomData,
  sync::mpsc::{channel, Receiver, Sender},
  thread::{self, JoinHandle},
  time::{Duration, Instant},
};

/// A send only PHY layer object.  
/// - PG: preamble generator
/// - MM: modulator/demodulator
/// - SS: sample input stream
/// - E: sample input stream error type
pub struct PhySender<PG, MM, SS, E, const PAD: usize = { DefaultConfig::SENDER_PADDING }> {
  _pg: PhantomData<PG>,
  _err: PhantomData<E>,
  preamble_samples: Vec<FP>,
  modem: MM,
  stream_out: SS,
}

impl<PG, MM, SS, E, const PAD: usize> PhySender<PG, MM, SS, E, PAD>
where
  PG: PreambleGen,
  MM: Modem,
  SS: OutStream<FP, E>,
{
  pub fn new(stream_out: SS, modem: MM) -> Self {
    let preamble_samples = PG::generate().samples();

    Self {
      _pg: PhantomData::default(),
      _err: PhantomData::default(),
      preamble_samples,
      modem,
      stream_out,
    }
  }

  pub const SAMPLES_PER_PACKET: usize = PG::PREAMBLE_LEN + MM::SAMPLES_PER_PACKET;
}
impl<PG, MM, SS, E, const PAD: usize> PacketSender<PhyPacket, E> for PhySender<PG, MM, SS, E, PAD>
where
  PG: PreambleGen,
  MM: Modem,
  SS: OutStream<FP, E>,
{
  /// frame = warm up + preamble + payload  
  /// - warm up: random samples whose absolute value is cloes to 1.0
  /// - preamble: predefined samples
  /// - payload: output of modulation on packet bytes
  /// NOTE: write them to the underlying stream together with `write_once`
  fn send(&mut self, packet: PhyPacket) -> Result<(), E> {
    assert_eq!(packet.len(), MM::BYTES_PER_PACKET);
    let mut buf = Vec::with_capacity(PG::PREAMBLE_LEN + MM::SAMPLES_PER_PACKET);
    buf.extend(&self.preamble_samples);
    buf.extend(self.modem.modulate(&packet));
    buf.extend([FP::ZERO; PAD]);
    self.stream_out.write_exact(&buf)
  }
}

/// A receive only PHY layer object.  
/// - PG: preamble generator
/// - MM: modulation encoder/decoder
/// - FD: frame detector
/// - SS: sample output stream
/// - E: sample output stream error type
pub struct PhyReceiver<PG, MM, FD, SS, E> {
  _pg: PhantomData<PG>,
  _fd: PhantomData<FD>,
  _ss: PhantomData<SS>,
  _err: PhantomData<E>,
  modem: MM,
  frame_payload_rx: Receiver<FramePayload>,
  exit_tx: Sender<()>,
  handler: Option<JoinHandle<()>>,
}

impl<PG, MM, FD, SS, E> PhyReceiver<PG, MM, FD, SS, E>
where
  PG: PreambleGen,
  MM: Modem,
  FD: FrameDetector + Send + 'static,
  SS: InStream<FP, E> + Send + 'static,
  E: std::fmt::Debug,
{
  /// A separated worker thread repeatedly do the procedure
  /// 0. exit if notified by exit channel
  /// 1. fetch samples from underlying stream
  /// 2. push them to frame detector
  /// 3. if a frame is detected, send it to the PhyReceiver through a channel
  fn worker(mut stream_in: SS, mut frame_detector: FD, frame_playload_rx: Sender<FramePayload>, exit_rx: Receiver<()>) {
    // TODO: select a proper interval
    let fetch_interval =
      Duration::from_secs_f32(8.0 * DefaultConfig::BUFFER_SIZE as f32 / DefaultConfig::SAMPLE_RATE as f32);
    let last_fetch = Instant::now() - fetch_interval;
    // TODO: select a proper buffer size
    let mut buf = [Sample::ZERO; DefaultConfig::BUFFER_SIZE * 8];
    while exit_rx.try_recv().is_err() {
      if last_fetch.elapsed() > fetch_interval {
        let n = stream_in.read(&mut buf).unwrap();
        buf[..n].iter().for_each(|x| {
          if let Some(payload) = frame_detector.on_sample(*x) {
            frame_playload_rx.send(payload).unwrap();
          }
        });
      }
      thread::yield_now();
    }
  }

  pub fn new(stream_in: SS, modem: MM, frame_detector: FD) -> Self {
    let (exit_tx, exit_rx) = channel();
    let (frame_playload_tx, frame_payload_rx) = channel();
    let handler = thread::spawn(move || Self::worker(stream_in, frame_detector, frame_playload_tx, exit_rx));
    Self {
      _pg: PhantomData::default(),
      _fd: PhantomData::default(),
      _ss: PhantomData::default(),
      _err: PhantomData::default(),
      modem,
      frame_payload_rx,
      exit_tx,
      handler: Some(handler),
    }
  }
}

impl<PG, MM, FD, SS, E> PacketReceiver<PhyPacket, ()> for PhyReceiver<PG, MM, FD, SS, E>
where
  PG: PreambleGen,
  MM: Modem,
  FD: FrameDetector,
  SS: InStream<FP, E>,
{
  // receive frame from the channel and then demodulate the signal
  fn recv(&mut self) -> Result<PhyPacket, ()> {
    match self.frame_payload_rx.try_recv() {
      Ok(payload) => Ok(self.modem.demodulate(&payload)),
      Err(_) => Err(()),
    }
  }

  fn recv_timeout(&mut self, timeout: Duration) -> Result<PhyPacket, ()> {
    match self.frame_payload_rx.recv_timeout(timeout) {
      Ok(payload) => Ok(self.modem.demodulate(&payload)),
      Err(_) => Err(()),
    }
  }
}

impl<PG, MM, FD, SS, E> Drop for PhyReceiver<PG, MM, FD, SS, E> {
  // notify the worker thread to exit
  // wait for the worker thread to stop
  fn drop(&mut self) {
    self.exit_tx.send(()).unwrap();
    if let Some(worker) = self.handler.take() {
      worker.join().unwrap();
    }
  }
}

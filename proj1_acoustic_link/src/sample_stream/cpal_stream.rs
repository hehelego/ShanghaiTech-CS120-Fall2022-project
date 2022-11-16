use cpal::{
  traits::{DeviceTrait, HostTrait, StreamTrait},
  BuildStreamError, Device, StreamConfig,
};
use parking_lot::Mutex;
use std::{collections::VecDeque, sync::Arc};

use crate::{
  block_buffer::ConcurrentBuffer,
  traits::{InStream, OutStream, Sample, FP},
  DefaultConfig,
};

/// An input stream built on cpal input stream. Support reading PCM samples.
/// The `CpalInStream` fetch samples from a `cpal::Stream`
pub struct CpalInStream {
  stream: cpal::Stream,
  buffer: ConcurrentBuffer<FP>,
}
/// An output stream built on cpal output stream. Support writing PCM samples.
/// The `CpalOutStream` write samples to a `cpal::Stream`
pub struct CpalOutStream {
  stream: cpal::Stream,
  buffer: ConcurrentBuffer<FP>,
}
/// monitoring the power level on a cpal stream
pub struct CpalPowerProbe {
  _stream: cpal::Stream,
  power: Arc<Mutex<f32>>,
}

impl CpalInStream {
  /// Create an input stream on a given device with a specified config.
  /// The stream is initially in playing state.
  pub fn new(input_device: Device, stream_config: StreamConfig) -> Result<Self, BuildStreamError> {
    // the callback function  periodically fetch samples
    // from the stream and push them into the buffer
    let buffer: ConcurrentBuffer<FP> = Default::default();
    let mut bf = buffer.clone();
    let mut cast_buf = vec![FP::ZERO; DefaultConfig::BUFFER_SIZE];
    let stream = input_device.build_input_stream(
      &stream_config,
      move |data: &[f32], _: &_| CpalInStream::read_from_stream(data, &mut cast_buf, &mut bf),
      |err| eprintln!("An error occured at cpal stream {}", err),
    )?;
    Ok(CpalInStream { stream, buffer })
  }
  /// Start the stream. Change its state to playing and accept samples.
  pub fn play(&self) {
    self.stream.play().unwrap();
  }
  /// Pause the stream. All the samples come in when stream is paused will be discarded silently.
  pub fn pause(&self) {
    self.stream.pause().unwrap();
  }

  // the helper function passed to the `stream.build_input_stream`
  fn read_from_stream(data: &[f32], cast_buf: &mut [FP], dest: &mut ConcurrentBuffer<FP>) {
    let cast_buf = &mut cast_buf[..data.len()];
    cast_buf
      .iter_mut()
      .zip(data.iter())
      .for_each(|(x, y)| *x = FP::from_f32(*y));
    dest.write(cast_buf).unwrap();
  }
}

impl CpalOutStream {
  /// create an output stream on a given device with a specified config.
  /// The stream is initially in playing state.
  pub fn new(output_device: Device, stream_config: StreamConfig) -> Result<Self, BuildStreamError> {
    // the callback function should periodically fetch samples
    // from the buffer and write them into the stream
    let buffer: ConcurrentBuffer<FP> = Default::default();
    let mut bf = buffer.clone();
    let mut cast_buf = vec![FP::ZERO; DefaultConfig::BUFFER_SIZE];

    let stream = output_device.build_output_stream(
      &stream_config,
      move |data: &mut [f32], _| CpalOutStream::write_to_stream(data, &mut cast_buf, &mut bf),
      |e| eprintln!("An error occured at cpal out stream {}", e),
    )?;
    Ok(CpalOutStream { stream, buffer })
  }

  /// Start the stream. Change its state to playing and accept samples.
  pub fn play(&self) {
    self.stream.play().unwrap();
  }
  /// Pause the stream.
  pub fn pause(&self) {
    self.stream.pause().unwrap();
  }
  /// Clear the samples not played.
  pub fn clear(&mut self) {
    self.buffer.clear()
  }

  // helper function passed to the `stream.build_output_stream`
  fn write_to_stream(data: &mut [f32], cast_buf: &mut [FP], src: &mut ConcurrentBuffer<FP>) {
    let cast_buf = &mut cast_buf[0..data.len()];
    let read_size = src.read(cast_buf).unwrap();
    data[..read_size]
      .iter_mut()
      .zip(cast_buf.iter())
      .for_each(|(x, y)| *x = FP::into_f32(*y));
    data[read_size..].iter_mut().for_each(|x| *x = 0.0);
  }
}

impl CpalPowerProbe {
  /// create and start to listen on a input stream
  pub fn new(input_device: Device, stream_config: StreamConfig) -> Result<Self, BuildStreamError> {
    let power = Arc::new(Mutex::new(0.0));
    let pw = power.clone();
    let mut power_queue = VecDeque::with_capacity(DefaultConfig::PWR_PROBE_WIND);
    let _stream = input_device.build_input_stream(
      &stream_config,
      move |data: &[f32], _| {
        let mut pw = pw.lock();
        data.iter().for_each(|&x| {
          if power_queue.len() == DefaultConfig::PWR_PROBE_WIND {
            let y = power_queue.pop_front().unwrap();
            *pw -= y * y;
          }
          power_queue.push_back(x);
          *pw += x * x;
        });
      },
      |e| eprintln!("An error occured at cpal input stream {}", e),
    )?;
    Ok(CpalPowerProbe { _stream, power })
  }
  /// probe the power on the input stream
  pub fn power(&self) -> f32 {
    let power = self.power.lock();
    *power / DefaultConfig::PWR_PROBE_WIND as f32
  }
}

impl Default for CpalInStream {
  /// Build CpalInStream with default settings:
  /// - Channels: 1
  /// - Sample rate: 48000
  /// - buffer size: 1024
  fn default() -> Self {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("no default input device available");
    let stream_config = DefaultConfig::new();
    Self::new(input_device, stream_config).expect("failed to create input stream")
  }
}

impl InStream<FP, ()> for CpalInStream {
  /// Read as many as possible data from the stream and return immediatley with the number of samples read.
  fn read(&mut self, buf: &mut [FP]) -> Result<usize, ()> {
    self.buffer.read(buf)
  }

  /// Read exactly `buf.len()` samples from the stream.
  /// This function will not return untill all the samples have been read.
  fn read_exact(&mut self, buf: &mut [FP]) -> Result<(), ()> {
    self.buffer.read_exact(buf)
  }
}

impl Default for CpalOutStream {
  /// Build CpalOutStream with default settings:
  /// - Channels: 1
  /// - Sample rate: 48000
  /// - buffer size: 1024
  fn default() -> Self {
    let host = cpal::default_host();
    let output_device = host
      .default_output_device()
      .expect("no default output device available");
    let stream_config = DefaultConfig::new();
    Self::new(output_device, stream_config).expect("failed to create input stream")
  }
}

impl OutStream<FP, ()> for CpalOutStream {
  /// Write as many as possible data to the stream, and return with the number of samples written immediately.
  fn write(&mut self, buf: &[FP]) -> Result<usize, ()> {
    self.buffer.write(buf)
  }

  /// Write exactly `buf.len()` samples to the stream. This function will not return until all the samples are written.
  fn write_exact(&mut self, buf: &[FP]) -> Result<(), ()> {
    self.buffer.write_exact(buf)
  }

  /// forward to `ConcurrentBuffer::wait`
  fn wait(&mut self) {
    self.buffer.wait()
  }
}

impl Default for CpalPowerProbe {
  fn default() -> Self {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("no default input device available");
    let stream_config = DefaultConfig::new();
    Self::new(input_device, stream_config).expect("failed to create input stream")
  }
}

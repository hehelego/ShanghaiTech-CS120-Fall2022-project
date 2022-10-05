use cpal::{
  traits::{DeviceTrait, HostTrait, StreamTrait},
  BuildStreamError, Device, StreamConfig,
};

use crate::{
  block_buffer::ConcurrentBuffer,
  defaut_config,
  traits::{InStream, OutStream},
};

/// An input stream built on cpal input stream. Support reading PCM samples.
/// The `CpalInStream` fetch samples from a `cpal::Stream`
pub struct CpalInStream {
  stream: cpal::Stream,
  buffer: ConcurrentBuffer<f32>,
}
/// An output stream built on cpal output stream. Support writing PCM samples.
/// The `CpalOutStream` write samples to a `cpal::Stream`
pub struct CpalOutStream {
  stream: cpal::Stream,
  buffer: ConcurrentBuffer<f32>,
}

impl CpalInStream {
  /// Create an input stream on a given device with a specified config.
  /// The stream is initially in playing state.
  pub fn new(input_device: Device, stream_config: StreamConfig) -> Result<Self, BuildStreamError> {
    // the callback function  periodically fetch samples
    // from the stream and push them into the buffer
    let buffer: ConcurrentBuffer<f32> = Default::default();
    let mut bf = buffer.clone();
    let stream = input_device.build_input_stream(
      &stream_config,
      move |data: &[f32], _: &_| CpalInStream::read_from_stream(data, &mut bf),
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
  fn read_from_stream(data: &[f32], dest: &mut ConcurrentBuffer<f32>) {
    dest.write(data).unwrap();
  }
}

impl CpalOutStream {
  /// create an output stream on a given device with a specified config.
  /// The stream is initially in playing state.
  pub fn new(output_device: Device, stream_config: StreamConfig) -> Result<Self, BuildStreamError> {
    // the callback function should periodically fetch samples
    // from the buffer and write them into the stream
    let buffer: ConcurrentBuffer<f32> = Default::default();
    let mut bf = buffer.clone();

    let stream = output_device.build_output_stream(
      &stream_config,
      move |data: &mut [f32], _| CpalOutStream::write_to_stream(data, &mut bf),
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
  pub fn clear(&self) {
    self.buffer.clear()
  }

  // helper function passed to the `stream.build_output_stream`
  fn write_to_stream(data: &mut [f32], src: &mut ConcurrentBuffer<f32>) {
    let read_size = src.read(data).unwrap();
    data[read_size..].iter_mut().for_each(|x| *x = 0.0);
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
    let stream_config = StreamConfig {
      channels: defaut_config::CHANNELS,
      sample_rate: cpal::SampleRate(defaut_config::SAMPLE_RATE),
      buffer_size: cpal::BufferSize::Fixed(defaut_config::BUFFER_SIZE as u32),
    };
    Self::new(input_device, stream_config).expect("failed to create input stream")
  }
}

impl InStream<f32, ()> for CpalInStream {
  /// Read as many as possible data from the stream and return immediatley with the number of samples read.
  fn read(&mut self, buf: &mut [f32]) -> Result<usize, ()> {
    self.buffer.read(buf)
  }

  /// Read exactly `buf.len()` samples from the stream.
  /// This function will not return untill all the samples have been read.
  fn read_exact(&mut self, buf: &mut [f32]) -> Result<(), ()> {
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
    let stream_config = StreamConfig {
      channels: defaut_config::CHANNELS,
      sample_rate: cpal::SampleRate(defaut_config::SAMPLE_RATE),
      buffer_size: cpal::BufferSize::Fixed(defaut_config::BUFFER_SIZE as u32),
    };
    Self::new(output_device, stream_config).expect("failed to create input stream")
  }
}

impl OutStream<f32, ()> for CpalOutStream {
  /// Write as many as possible data to the stream, and return with the number of samples written immediately.
  fn write(&mut self, buf: &[f32]) -> Result<usize, ()> {
    self.buffer.write(buf)
  }

  /// Write exactly `buf.len()` samples to the stream. This function will not return until all the samples are written.
  fn write_exact(&mut self, buf: &[f32]) -> Result<(), ()> {
    self.buffer.write_exact(buf)
  }
}

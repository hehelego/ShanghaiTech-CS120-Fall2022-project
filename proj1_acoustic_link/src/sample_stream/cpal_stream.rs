use cpal::{
  traits::{DeviceTrait, HostTrait, StreamTrait},
  BuildStreamError, Device, StreamConfig,
};

use crate::{
  block_buffer::ConcurrentBuffer,
  traits::{InStream, OutStream},
};

/// An input stream built on cpal input stream. Support reading PCM samples.
pub struct CpalInStream {
  stream: cpal::Stream,
  buffer: ConcurrentBuffer<f32>,
}
/// An output stream built on cpal output stream. Support writing PCM samples.
pub struct CpalOutStream {
  stream: cpal::Stream,
  buffer: ConcurrentBuffer<f32>,
}

impl CpalInStream {
  /// create an input stream on a given device with a specified config.
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

  pub fn play(&self) {
    self.stream.play().unwrap();
  }

  pub fn pause(&self) {
    self.stream.pause().unwrap();
  }

  fn read_from_stream(data: &[f32], dest: &mut ConcurrentBuffer<f32>) {
    dest.write(data).unwrap();
  }
}

impl CpalOutStream {
  /// create an output stream on a given device with a specified config.
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

  pub fn pause(&self) {
    self.stream.pause().unwrap();
  }
  pub fn play(&self) {
    self.stream.play().unwrap();
  }

  fn write_to_stream(data: &mut [f32], src: &mut ConcurrentBuffer<f32>) {
    let read_size = src.read(data).unwrap();
    data[read_size..].iter_mut().for_each(|x| *x = 0.0);
  }
}

impl Default for CpalInStream {
  fn default() -> Self {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("no default input device available");
    let stream_config = StreamConfig {
      channels: 1,
      sample_rate: cpal::SampleRate(48000),
      buffer_size: cpal::BufferSize::Fixed(1024),
    };
    Self::new(input_device, stream_config).expect("failed to create input stream")
  }
}

impl InStream<f32, ()> for CpalInStream {
  fn read(&mut self, buf: &mut [f32]) -> Result<usize, ()> {
    self.buffer.read(buf)
  }

  fn read_exact(&mut self, buf: &mut [f32]) -> Result<(), ()> {
    self.buffer.read_exact(buf)
  }
}

impl Default for CpalOutStream {
  fn default() -> Self {
    let host = cpal::default_host();
    let output_device = host
      .default_output_device()
      .expect("no default output device available");
    let stream_config = StreamConfig {
      channels: 1,
      sample_rate: cpal::SampleRate(48000),
      buffer_size: cpal::BufferSize::Fixed(1024),
    };
    Self::new(output_device, stream_config).expect("failed to create input stream")
  }
}

impl OutStream<f32, ()> for CpalOutStream {
  fn write(&mut self, buf: &[f32]) -> Result<usize, ()> {
    self.buffer.write(buf)
  }

  fn write_exact(&mut self, buf: &[f32]) -> Result<(), ()> {
    self.buffer.write_exact(buf)
  }
}

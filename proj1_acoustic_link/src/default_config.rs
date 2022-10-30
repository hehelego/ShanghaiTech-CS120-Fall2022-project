use cpal::StreamConfig;
use hound::WavSpec;

pub struct DefaultConfig;
impl DefaultConfig {
  pub const CHANNELS: u16 = 1;
  pub const SAMPLE_RATE: u32 = 48000;
  pub const BUFFER_SIZE: usize = 1024;
  pub const BITS_PER_SAMPE: u16 = 32;
  pub const PHYTX_PAD_SAMPLES: usize = 20;

  pub fn new<T>() -> T
  where
    Self: Into<T>,
  {
    DefaultConfig.into()
  }
}

impl Into<StreamConfig> for DefaultConfig {
  fn into(self) -> StreamConfig {
    StreamConfig {
      channels: Self::CHANNELS,
      sample_rate: cpal::SampleRate(Self::SAMPLE_RATE),
      buffer_size: cpal::BufferSize::Fixed(Self::BUFFER_SIZE as u32),
    }
  }
}

impl Into<WavSpec> for DefaultConfig {
  fn into(self) -> WavSpec {
    WavSpec {
      channels: Self::CHANNELS,
      sample_rate: Self::SAMPLE_RATE,
      bits_per_sample: Self::BITS_PER_SAMPE,
      sample_format: hound::SampleFormat::Float,
    }
  }
}

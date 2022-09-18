use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BuildStreamError, Device, StreamConfig,
};

use crate::{
    block_buffer::ConcurrentBuffer,
    traits::{InStream, OutStream},
};

pub struct CpalInStream {
    _stream: cpal::Stream,
    buffer: ConcurrentBuffer<f32>,
}
pub struct CpalOutStream {
    _stream: cpal::Stream,
    buffer: ConcurrentBuffer<f32>,
}
pub struct CpalStreamPair {
    in_part: CpalInStream,
    out_part: CpalOutStream,
}

impl CpalInStream {
    pub fn new(
        input_device: Device,
        stream_config: StreamConfig,
    ) -> Result<Self, BuildStreamError> {
        todo!();
    }
}
impl CpalOutStream {
    pub fn new(
        output_device: Device,
        stream_config: StreamConfig,
    ) -> Result<Self, BuildStreamError> {
        todo!();
    }
}
impl CpalStreamPair {
    fn new(in_part: CpalInStream, out_part: CpalOutStream) -> Self {
        Self { in_part, out_part }
    }
}
impl Default for CpalInStream {
    fn default() -> Self {
        let host = cpal::default_host();
        let input_device = host
            .default_input_device()
            .expect("no default input device available");
        let stream_config = input_device
            .default_input_config()
            .expect("no default input config available")
            .config();

        Self::new(input_device, stream_config).expect("failed to create input stream")
    }
}
impl Default for CpalOutStream {
    fn default() -> Self {
        let host = cpal::default_host();
        let output_device = host
            .default_output_device()
            .expect("no default output device available");
        let stream_config = output_device
            .default_output_config()
            .expect("no default input config available")
            .config();

        Self::new(output_device, stream_config).expect("failed to create input stream")
    }
}
impl Default for CpalStreamPair {
    fn default() -> Self {
        Self::new(Default::default(), Default::default())
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
impl OutStream<f32, ()> for CpalOutStream {
    fn write(&mut self, buf: &[f32]) -> Result<usize, ()> {
        self.buffer.write(buf)
    }

    fn write_exact(&mut self, buf: &[f32]) -> Result<(), ()> {
        self.buffer.write_exact(buf)
    }
}
impl InStream<f32, ()> for CpalStreamPair {
    fn read(&mut self, buf: &mut [f32]) -> Result<usize, ()> {
        self.in_part.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [f32]) -> Result<(), ()> {
        self.in_part.read_exact(buf)
    }
}
impl OutStream<f32, ()> for CpalStreamPair {
    fn write(&mut self, buf: &[f32]) -> Result<usize, ()> {
        self.out_part.write(buf)
    }

    fn write_exact(&mut self, buf: &[f32]) -> Result<(), ()> {
        self.out_part.write_exact(buf)
    }
}

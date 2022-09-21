use std::io::{Read, Seek, Write};

use hound::{Error as WavError, WavReader, WavWriter};

use crate::traits::{InStream, OutStream};

impl<R: Read> InStream<f32, WavError> for WavReader<R> {
  fn read(&mut self, buf: &mut [f32]) -> Result<usize, WavError> {
    todo!()
  }

  fn read_exact(&mut self, buf: &mut [f32]) -> Result<(), WavError> {
    todo!()
  }
}
impl<W: Write + Seek> OutStream<f32, WavError> for WavWriter<W> {
  fn write(&mut self, buf: &[f32]) -> Result<usize, WavError> {
    todo!()
  }

  fn write_exact(&mut self, buf: &[f32]) -> Result<(), WavError> {
    todo!()
  }
}

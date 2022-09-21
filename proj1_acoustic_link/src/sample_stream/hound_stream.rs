use std::io::{Read, Seek, Write};

use hound::{Error as WavError, WavReader, WavWriter};

use crate::traits::{InStream, OutStream};

pub struct HoundInStream<R: Read>(WavReader<R>);

pub struct HoundOutStream<W: Write + Seek>(WavWriter<W>);

impl<R: Read> HoundInStream<R> {
  pub fn new(wav_reader: WavReader<R>) -> Self {
    Self(wav_reader)
  }
}
impl<W: Write + Seek> HoundOutStream<W> {
  pub fn new(wav_writer: WavWriter<W>) -> Self {
    Self(wav_writer)
  }
}

impl<R: Read> InStream<f32, WavError> for HoundInStream<R> {
  fn read(&mut self, buf: &mut [f32]) -> Result<usize, WavError> {
    let mut n = 0;
    for (x, sample) in buf.iter_mut().zip(self.0.samples()) {
      let sample = sample?;
      *x = sample;
      n += 1;
    }
    Ok(n)
  }

  fn read_exact(&mut self, buf: &mut [f32]) -> Result<(), WavError> {
    self.read(buf).map(|_| ())
  }
}
impl<W: Write + Seek> OutStream<f32, WavError> for HoundOutStream<W> {
  fn write(&mut self, buf: &[f32]) -> Result<usize, WavError> {
    let mut n = 0;
    for x in buf {
      self.0.write_sample(*x)?;
      n += 1;
    }
    Ok(n)
  }

  fn write_exact(&mut self, buf: &[f32]) -> Result<(), WavError> {
    self.write(buf).map(|_| ())
  }
}

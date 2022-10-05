use std::{
  fs::File,
  io::{BufReader, BufWriter, Read, Seek, Write},
  path::Path,
};

use hound::{Error as WavError, WavReader, WavWriter};

use crate::{
  defaut_config,
  traits::{InStream, OutStream},
};

pub struct HoundInStream<R: Read>(WavReader<R>);

pub struct HoundOutStream<W: Write + Seek>(WavWriter<W>);

impl<R: Read> HoundInStream<R> {
  pub fn new(wav_reader: WavReader<R>) -> Self {
    Self(wav_reader)
  }
}
impl HoundInStream<BufReader<File>> {
  pub fn open<P>(filename: P) -> HoundInStream<BufReader<File>>
  where
    P: AsRef<Path>,
  {
    HoundInStream::new(hound::WavReader::open(filename).unwrap())
  }
}

impl<W: Write + Seek> HoundOutStream<W> {
  pub fn new(wav_writer: WavWriter<W>) -> Self {
    Self(wav_writer)
  }

  pub fn finalize(self) {
    self.0.finalize().unwrap()
  }
}
impl HoundOutStream<BufWriter<File>> {
  pub fn create<P>(filename: P) -> HoundOutStream<BufWriter<File>>
  where
    P: AsRef<Path>,
  {
    let spec = hound::WavSpec {
      channels: defaut_config::CHANNELS,
      sample_rate: defaut_config::SAMPLE_RATE,
      bits_per_sample: defaut_config::BITS_PER_SAMPE,
      sample_format: hound::SampleFormat::Float,
    };
    HoundOutStream::new(hound::WavWriter::create(filename, spec).unwrap())
  }
}

impl<R: Read> InStream<f32, WavError> for HoundInStream<R> {
  fn read(&mut self, buf: &mut [f32]) -> Result<usize, WavError> {
    let mut n = 0;
    for (x, sample) in buf.iter_mut().zip(self.0.samples()) {
      *x = sample?;
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

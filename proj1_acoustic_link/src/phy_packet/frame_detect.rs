use cpal::Sample;

use super::{FrameDetector, FramePayload, PreambleGen};
use crate::helper::dot_product;
use std::collections::VecDeque;

enum FramingState {
  DetectPreambleStart,
  DetectRisingEdge,
  DetectFallingEdge,
  WaitPayload,
}

struct PreambleWindow {
  buffer: VecDeque<f32>,
  head_index: usize,
  tail_index: usize,
  capacity: usize,
  smooth_power: f32,
  square_sum: f32,
}

impl PreambleWindow {
  pub fn with_capacity(capacity: usize) -> PreambleWindow {
    Self {
      buffer: VecDeque::new(),
      head_index: 0,
      tail_index: 0,
      smooth_power: 0.0,
      square_sum: 0.0,
      capacity,
    }
  }
  pub fn len(&self) -> usize {
    self.buffer.len()
  }
  pub fn iter(&self) -> std::collections::vec_deque::Iter<f32> {
    self.buffer.iter()
  }
  pub fn norm(&self) -> f32 {
    self.square_sum.sqrt()
  }
  pub fn update(&mut self, sample: f32) {
    self.tail_index += 1;
    self.buffer.push_back(sample);
    self.square_sum += sample * sample;
    self.smooth_power = self.smooth_power * 63.0 / 64.0 + sample * sample / 64.0;
    // Pop steal samples
    if self.buffer.len() > self.capacity {
      let old_sample = self.buffer.pop_front().unwrap();
      self.head_index += 1;
      self.square_sum -= old_sample * old_sample;
    }
  }
  pub fn extract_samples_to_end(&mut self, index: usize) -> Vec<f32> {
    let start = index - self.head_index + 1;
    self.buffer.range(start..).cloned().collect()
  }

  pub fn clear(&mut self) {
    self.buffer.clear();
    self.head_index = 0;
    self.tail_index = 0;
    self.smooth_power = 0.0;
    self.square_sum = 0.0;
  }
}

pub struct CorrelationFraming {
  detect_window: PreambleWindow,
  state: FramingState,
  frame_payload: Vec<f32>,
  payload_size: usize,
  peak_index: usize,
  peak_value: f32,
  // part: preamble
  preamble_len: usize, // number of samples in the preamble sequence
  preamble: Vec<f32>,  // preamble sample sequence
  preamble_norm: f32,  // norm of the preamble vector: square root of the square sum of preamble samples
}

impl CorrelationFraming {
  /// For preamble detection,
  /// the ratio of the correlation power to the received signal average power
  /// must be greater than this threshold.
  /// Different threshold should be used for different media.
  /// - ideal transmission: 0.7
  /// - air gapped transmission: 2.0
  pub const CORR_TO_PWR_MIN: f32 = if cfg!(feature = "wired") { 0.7 } else { 2.0 };
  /// Minimum cosine similarity required for preamble detection.
  /// Used to skip irrelevant patterns with extremely high power.
  pub const COSINE_MIN: f32 = 0.4;
  /// The falling edge can be detected about 200 samples after the correlation peak appears.
  pub const AFTER_PEAK_SAMPLES: usize = 200;

  pub fn new<PG, const PAYLOAD_LEN: usize>() -> CorrelationFraming
  where
    PG: PreambleGen,
  {
    let preamble = PG::generate();
    let sqr_sum = preamble.iter().fold(0.0, |s, &x| s + x * x);
    println!("Detect Preamble Start");
    Self {
      detect_window: PreambleWindow::with_capacity(PG::PREAMBLE_LEN),
      state: FramingState::DetectPreambleStart,
      frame_payload: Vec::new(),
      payload_size: PAYLOAD_LEN,
      preamble_len: PG::PREAMBLE_LEN,
      preamble: PG::generate(),
      preamble_norm: sqr_sum.sqrt(),
      peak_index: 0,
      peak_value: 0.0,
    }
  }

  // detect the start of preamble. After the checks passed, it will enter detect rising edge state.
  fn detect_preamble_start(&mut self, sample: f32) -> FramingState {
    self.detect_window.update(sample);
    // Not enough samples
    if self.detect_window.len() < self.preamble_len {
      return FramingState::DetectPreambleStart;
    }
    // To check if is the beginning of the preamble
    let dot = dot_product(self.detect_window.iter(), self.preamble.iter());
    let corr2pwr = (dot / self.preamble_len as f32) / self.detect_window.smooth_power;
    let cosine_sim = dot / self.preamble_norm / self.detect_window.norm();
    if corr2pwr >= Self::CORR_TO_PWR_MIN && cosine_sim >= Self::COSINE_MIN {
      self.peak_value = corr2pwr;
      self.peak_index = self.detect_window.tail_index;
      println!("Log: Detect Rising Edge");
      return FramingState::DetectRisingEdge;
    }
    // Wait for preambles
    return FramingState::DetectPreambleStart;
  }

  // Try to find the peak of the preamble. If it starts falling, it will enter detect falling edge.
  fn detect_rising_edge(&mut self, sample: f32) -> FramingState {
    self.detect_window.update(sample);
    // Get the data.
    let dot = dot_product(self.detect_window.iter(), self.preamble.iter());
    let corr2pwr = (dot / self.preamble_len as f32) / self.detect_window.smooth_power;
    let cosine_sim = dot / self.preamble_norm / self.detect_window.norm();
    if corr2pwr < self.peak_value || cosine_sim < Self::COSINE_MIN {
      println!("Log: Detect Falling Edge");
      return FramingState::DetectFallingEdge;
    }
    self.peak_value = corr2pwr;
    self.peak_index = self.detect_window.tail_index;
    return FramingState::DetectRisingEdge;
  }

  fn detect_falling_edge(&mut self, sample: f32) -> FramingState {
    self.detect_window.update(sample);
    // If we have found the tail.
    if self.detect_window.tail_index - self.peak_index > Self::AFTER_PEAK_SAMPLES {
      self.frame_payload = self.detect_window.extract_samples_to_end(self.peak_index);
      self.detect_window.clear();
      self.peak_value = 0.0;
      self.peak_index = 0;
      println!("Log: Wait Payload");
      return FramingState::WaitPayload;
    }
    return FramingState::DetectFallingEdge;
  }

  fn wait_payload(&mut self, sample: f32) -> (FramingState, Option<FramePayload>) {
    // assert!(self.frame_payload.len() < self.frame_payload.capacity());
    // append the newly found sample into the payload window
    self.frame_payload.push(sample);
    // wait until we have enough example
    if self.frame_payload.len() == self.payload_size {
      // extract the payload section, clear the payload window.
      let frame = self.frame_payload.clone();
      self.frame_payload.clear();
      // we have extracted a whole frame,
      // next: try to detect another frame
      (FramingState::DetectPreambleStart, Some(frame))
    } else {
      // no enough samples, continue to wait for more samples.
      (FramingState::WaitPayload, None)
    }
  }
}

impl FrameDetector for CorrelationFraming {
  fn on_sample(&mut self, sample: f32) -> Option<FramePayload> {
    match self.state {
      FramingState::WaitPayload => {
        let (state, frame) = self.wait_payload(sample);
        self.state = state;
        frame
      }
      FramingState::DetectPreambleStart => {
        self.state = self.detect_preamble_start(sample);
        None
      }
      FramingState::DetectRisingEdge => {
        self.state = self.detect_rising_edge(sample);
        None
      }
      FramingState::DetectFallingEdge => {
        self.state = self.detect_falling_edge(sample);
        None
      }
    }
  }
}

#[cfg(test)]
mod tests {

  use crate::phy_packet::{preambles::ChirpUpDown, FrameDetector, PreambleGen};

  use super::CorrelationFraming;

  #[test]
  fn corr_detect() {
    const PL_LEN: usize = 500;

    let preamble = ChirpUpDown::generate();
    let mut detector = CorrelationFraming::new::<ChirpUpDown, PL_LEN>();

    // preamble sequence
    preamble
      .iter()
      .map(|x| x * 0.8 + 0.1 * x.sin())
      .for_each(|x| assert_eq!(detector.on_sample(x), None));

    // the payload: exactly one frame will be found
    let mut s = 0;
    for i in 0..PL_LEN * 2 {
      let v = (0.33 * i as f32).sin();
      if detector.on_sample(v).is_some() {
        println!("detected {}/{}", i, PL_LEN);
        s += 1;
      }
    }
    assert_eq!(s, 1);

    // trash: no frame will be found
    s = 0;
    for i in 0..PL_LEN * 2 {
      let v = (0.33 * i as f32).sin();
      if detector.on_sample(v).is_some() {
        println!("detected {}/{}", i, PL_LEN);
        s += 1;
      }
    }
    assert_eq!(s, 0);
  }

  #[test]
  fn corr_detect_none() {
    const PRE_LEN: usize = 200;
    const PL_LEN: usize = 400;

    let mut detector = CorrelationFraming::new::<ChirpUpDown, PL_LEN>();

    // trash sequence
    let recv: Vec<_> = (0..PRE_LEN).map(|x| x as f32).collect();
    recv.iter().for_each(|&x| assert_eq!(detector.on_sample(x), None));

    // more random stuff
    let mut s = 0;
    for i in 0..PL_LEN * 20 {
      let v = (0.33 * i as f32).sin();
      if detector.on_sample(v).is_some() {
        println!("detected {}/{}", i, PL_LEN);
        s += 1;
      }
    }
    // expectation: no frame found
    assert_eq!(s, 0);
  }
}

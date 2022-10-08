use super::{FrameDetector, FramePayload, PreambleGen};
use crate::helper::dot_product;
use std::collections::VecDeque;

// State for CorraltionFraming
enum FramingState {
  DetectPreambleStart,
  DetectRisingEdge,
  DetectFallingEdge,
  WaitPayload,
}

// Window for store the samples temporarily.
struct PreambleWindow {
  buffer: VecDeque<f32>,
  // The two index will continous increase during one detection session, and reset to zero after detecting one package.
  head_index: usize,
  tail_index: usize,
  capacity: usize,
  smooth_power: f32,
  square_sum: f32,
}

impl PreambleWindow {
  // create PreambleWindow capacity. This should be at least as long as the preamble.
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
  pub fn iter(&self) -> impl ExactSizeIterator<Item = &'_ f32> {
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
    // Pop the steal sample
    if self.buffer.len() > self.capacity {
      let old_sample = self.buffer.pop_front().unwrap();
      self.head_index += 1;
      self.square_sum -= old_sample * old_sample;
    }
  }
  // Extract the samples from `index + 1` to the end of the buffer.
  pub fn extract_samples_to_end(&mut self, index: usize) -> Vec<f32> {
    let start = index - self.head_index;
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

struct Payload {
  buffer: Vec<f32>,
  size: usize,
}

impl Payload {
  pub fn new<const SIZE: usize>() -> Self {
    Self {
      buffer: Vec::with_capacity(SIZE),
      size: SIZE,
    }
  }
  pub fn extend(&mut self, samples: Vec<f32>) {
    assert!(samples.len() <= self.size);
    self.buffer.extend(samples.iter())
  }
  pub fn update(&mut self, sample: f32) -> Option<Vec<f32>> {
    self.buffer.push(sample);
    if self.buffer.len() == self.size {
      let frame = self.buffer.clone();
      self.buffer.clear();
      Some(frame)
    } else {
      None
    }
  }
}

pub struct CorrelationFraming<PG: PreambleGen> {
  state: FramingState,
  preamble_gen: PG,
  detect_window: PreambleWindow,
  frame_payload: Payload,
  corr_peak_index: usize,
  corr_peak_value: f32,
}

impl<PG> CorrelationFraming<PG>
where
  PG: PreambleGen,
{
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

  /// Create the CorrelationFraming detector with given preamble generator and layload length
  pub fn new<const PAYLOAD_LEN: usize>(preamble_gen: PG) -> CorrelationFraming<PG>
  where
    PG: PreambleGen,
  {
    println!("Detect Preamble Start");
    Self {
      detect_window: PreambleWindow::with_capacity(PG::PREAMBLE_LEN),
      state: FramingState::DetectPreambleStart,
      frame_payload: Payload::new::<PAYLOAD_LEN>(),
      corr_peak_index: 0,
      corr_peak_value: 0.0,
      preamble_gen,
    }
  }

  // detect the start of preamble. After the checks passed, it will enter detect rising edge state.
  fn detect_preamble_start(&mut self, sample: f32) -> FramingState {
    self.detect_window.update(sample);
    // Not enough samples
    if self.detect_window.len() < self.preamble_gen.len() {
      return FramingState::DetectPreambleStart;
    }
    // To check if is the beginning of the preamble
    let (corr2pwr, cosine_sim) = self.calculate_relations();
    if corr2pwr >= Self::CORR_TO_PWR_MIN && cosine_sim >= Self::COSINE_MIN {
      self.corr_peak_value = corr2pwr;
      self.corr_peak_index = self.detect_window.tail_index;
      println!("Log: Detect Rising Edge");
      FramingState::DetectRisingEdge
    } else {
      // Wait for preambles
      FramingState::DetectPreambleStart
    }
  }

  // Try to find the peak of the preamble. If it starts falling, it will enter detect falling edge.
  fn detect_rising_edge(&mut self, sample: f32) -> FramingState {
    self.detect_window.update(sample);
    // Get the data.
    let (corr2pwr, cosine_sim) = self.calculate_relations();
    if corr2pwr < self.corr_peak_value || cosine_sim < Self::COSINE_MIN {
      println!("Log: Detect Falling Edge");
      return FramingState::DetectFallingEdge;
    }
    self.corr_peak_value = corr2pwr;
    self.corr_peak_index = self.detect_window.tail_index;
    FramingState::DetectRisingEdge
  }

  fn detect_falling_edge(&mut self, sample: f32) -> FramingState {
    self.detect_window.update(sample);
    // If we have found the tail.
    if self.detect_window.tail_index - self.corr_peak_index > Self::AFTER_PEAK_SAMPLES {
      self
        .frame_payload
        .extend(self.detect_window.extract_samples_to_end(self.corr_peak_index));
      println!("Log: Wait Payload. Peak index: {}", self.corr_peak_index);
      self.reset_detection_state();
      println!("Log: Wait Payload");
      FramingState::WaitPayload
    } else {
      FramingState::DetectFallingEdge
    }
  }

  fn wait_payload(&mut self, sample: f32) -> (FramingState, Option<FramePayload>) {
    // assert!(self.frame_payload.len() < self.frame_payload.capacity());
    // append the newly found sample into the payload window
    if let Some(payload) = self.frame_payload.update(sample) {
      (FramingState::DetectPreambleStart, Some(payload))
    } else {
      (FramingState::WaitPayload, None)
    }
    // wait until we have enough example
  }

  // Calculate the relations between preamble and samples. Return the ratio of correlation power to the received signal average power and cosine similarity.
  fn calculate_relations(&self) -> (f32, f32) {
    let dot = dot_product(self.detect_window.iter(), self.preamble_gen.iter());
    let corr2pwr = (dot / self.preamble_gen.len() as f32) / self.detect_window.smooth_power;
    let cosine_sim = dot / self.preamble_gen.norm() / self.detect_window.norm();
    (corr2pwr, cosine_sim)
  }

  // reset the fields relatated to preable detection.
  fn reset_detection_state(&mut self) {
    self.detect_window.clear();
    self.corr_peak_value = 0.0;
    self.corr_peak_index = 0;
  }
}

impl<PG> FrameDetector for CorrelationFraming<PG>
where
  PG: PreambleGen,
{
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
mod tests;

use super::{FrameDetector, FramePayload, PreambleGen};
use crate::helper::dot_product;
use crate::traits::{Sample, FP};
use std::collections::VecDeque;

// State for CorraltionFraming
enum FramingState {
  DetectPreambleStart,
  DetectRisingEdge,
  WaitPayload,
}

// Window for store the samples temporarily.
struct PreambleWindow {
  pub(self) buffer: VecDeque<FP>,
  // The two index will continous increase during one detection session, and reset to zero after detecting one package.
  head_index: usize,
  tail_index: usize,
  capacity: usize,
  smooth_power: FP,
  square_sum: FP,
}

impl PreambleWindow {
  // create PreambleWindow capacity. This should be at least as long as the preamble.
  pub fn with_capacity(capacity: usize) -> PreambleWindow {
    Self {
      buffer: VecDeque::new(),
      head_index: 0,
      tail_index: 0,
      smooth_power: FP::ZERO,
      square_sum: FP::ZERO,
      capacity,
    }
  }
  pub fn len(&self) -> usize {
    self.buffer.len()
  }

  pub fn update(&mut self, sample: FP) {
    self.tail_index += 1;
    self.buffer.push_back(sample);
    self.square_sum += sample * sample;
    self.smooth_power = self.smooth_power * FP::from_f32(31.0 / 32.0) + sample * sample / FP::from_f32(32.0);
    // Pop the steal sample
    if self.buffer.len() > self.capacity {
      let old_sample = self.buffer.pop_front().unwrap();
      self.head_index += 1;
      self.square_sum -= old_sample * old_sample;
    }
  }
  // Extract the samples from `index + 1` to the end of the buffer.
  pub fn extract_samples_to_end(&mut self, index: usize) -> Vec<FP> {
    assert!(index >= self.head_index);
    let start = index - self.head_index;
    assert!(start < self.buffer.len());
    self.buffer.range(start..).cloned().collect()
  }
  pub fn clear(&mut self) {
    self.buffer.clear();
    self.head_index = 0;
    self.tail_index = 0;
    self.smooth_power = FP::ZERO;
    self.square_sum = FP::ZERO;
  }

  pub fn enough_power(&self) -> bool {
    self.smooth_power > FP::ZERO && self.square_sum > FP::ZERO
  }
}

struct Payload {
  buffer: Vec<FP>,
  size: usize,
}

impl Payload {
  pub fn new<const SIZE: usize>() -> Self {
    Self {
      buffer: Vec::with_capacity(SIZE),
      size: SIZE,
    }
  }
  pub fn extend(&mut self, samples: Vec<FP>) {
    assert!(samples.len() <= self.size);
    self.buffer.extend(samples.iter())
  }
  pub fn update(&mut self, sample: FP) -> Option<Vec<FP>> {
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
  corr_peak_value: FP,
}

impl<PG> CorrelationFraming<PG>
where
  PG: PreambleGen,
{
  /// For preamble detection,
  /// the ratio of the correlation power to the received signal average power
  /// must be greater than this threshold.
  pub const CORR_MIN: f32 = 3.0;
  /// The falling edge can be detected about 70 samples after the correlation peak appears.
  pub const AFTER_PEAK_SAMPLES: usize = 200;
  /// The minimum power required in detect window when detecting preamble
  /// Used to skip irrelevant patterns with extremely low power.
  /// **TODO** Currently only enabled for wired connection, find a proper value for wireless case
  /// **TODO** Find a proper value and proper volume configuration
  pub const POWER_MIN: f32 = if cfg!(feature = "wired") { 0.05 / 64.0 } else { 0.0 };

  /// Create the CorrelationFraming detector with given preamble generator and layload length
  pub fn new<const PAYLOAD_LEN: usize>(preamble_gen: PG) -> CorrelationFraming<PG>
  where
    PG: PreambleGen,
  {
    Self {
      detect_window: PreambleWindow::with_capacity(PG::PREAMBLE_LEN + Self::AFTER_PEAK_SAMPLES),
      state: FramingState::DetectPreambleStart,
      frame_payload: Payload::new::<PAYLOAD_LEN>(),
      corr_peak_index: 0,
      corr_peak_value: FP::ZERO,
      preamble_gen,
    }
  }

  // detect the start of preamble. After the checks passed, it will enter detect rising edge state.
  fn detect_preamble_start(&mut self, sample: FP) -> FramingState {
    self.detect_window.update(sample);
    // Not enough samples
    if self.detect_window.len() < PG::PREAMBLE_LEN || !self.detect_window.enough_power() {
      return FramingState::DetectPreambleStart;
    }
    // To check if is the beginning of the preamble
    let corr = self.corr();
    let pwr = self.detect_window.smooth_power;
    if corr.into_f32() > Self::CORR_MIN && pwr.into_f32() > Self::POWER_MIN {
      self.corr_peak_value = corr;
      self.corr_peak_index = self.detect_window.tail_index;
      FramingState::DetectRisingEdge
    } else {
      // Wait for preambles
      FramingState::DetectPreambleStart
    }
  }

  // Try to find the peak of the preamble. If it starts falling, it will enter detect falling edge.
  fn detect_rising_edge(&mut self, sample: FP) -> FramingState {
    self.detect_window.update(sample);
    // Get the data.
    let corr = self.corr();
    if corr > self.corr_peak_value {
      self.corr_peak_value = corr;
      self.corr_peak_index = self.detect_window.tail_index;
    } else if self.detect_window.tail_index - self.corr_peak_index > Self::AFTER_PEAK_SAMPLES {
      self
        .frame_payload
        .extend(self.detect_window.extract_samples_to_end(self.corr_peak_index));
      self.reset_detection_state();
      return FramingState::WaitPayload;
    }
    FramingState::DetectRisingEdge
  }

  fn wait_payload(&mut self, sample: FP) -> (FramingState, Option<FramePayload>) {
    // append the newly found sample into the payload window
    if let Some(payload) = self.frame_payload.update(sample) {
      (FramingState::DetectPreambleStart, Some(payload))
    } else {
      (FramingState::WaitPayload, None)
    }
    // wait until we have enough example
  }

  // Calculate the relations between preamble and samples. Return the ratio of correlation power to the received signal average power and cosine similarity.
  fn corr(&self) -> FP {
    let r = self.detect_window.len();
    dot_product(
      self.detect_window.buffer.range(r - PG::PREAMBLE_LEN..),
      self.preamble_gen.iter(),
    )
  }

  // reset the fields relatated to preable detection.
  fn reset_detection_state(&mut self) {
    self.detect_window.clear();
    self.corr_peak_value = FP::ZERO;
    self.corr_peak_index = 0;
  }
}

impl<PG> FrameDetector for CorrelationFraming<PG>
where
  PG: PreambleGen,
{
  fn on_sample(&mut self, sample: FP) -> Option<FramePayload> {
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
    }
  }
}

#[cfg(test)]
mod tests;

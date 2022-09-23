use super::{Frame, FrameDetector};
use crate::helper::dot_product;
use std::collections::VecDeque;

enum FramingState {
  DetectPreamble,
  WaitPayload,
}

/// If a preamble exists, the correlation pattern is:
/// 1. first goes up quickly when the preamble starts
/// 2. stay at plateau for a while
/// 3. reduce quickly after the preamble ends
/// Our approach is to detect **the falling edge**
pub struct CorrelationFraming {
  preamble_len: usize,
  payload_len: usize,
  preamble: Vec<f32>,
  preamble_norm: f32,
  state: FramingState,
  index: usize,
  stream_head: usize,
  power: f32,
  peak_value: f32,
  peak_index: usize,
  detect_window: VecDeque<f32>,
  detect_sqr_sum: f32,
  frame_window: VecDeque<f32>,
  stream_window: VecDeque<f32>,
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

  fn stream_range(&self, start: usize, end: usize) -> Vec<f32> {
    let offset = self.stream_head;
    let start = start - offset;
    let end = end - offset;
    self.stream_window.range(start..end).cloned().collect()
  }
  // stop when the Cosine-Similarity of detect window and the preamble is greater than a threshold
  fn iter_detect_preamble(&mut self, sample: f32) -> FramingState {
    // update the preamble detect window
    let head = self.detect_window.pop_front().unwrap();
    self.detect_window.push_back(sample);
    self.detect_sqr_sum += sample * sample - head * head;

    // compute the correlation
    let dot = dot_product(self.detect_window.iter(), self.preamble.iter());
    let corr2pwr = (dot / self.preamble_len as f32) / self.power;
    let cosine_sim = dot / self.preamble_norm / self.detect_sqr_sum.sqrt();

    // - max correlation: end of the preamble sequence
    // - correlation drops down: payload section begins
    // - otherwise: skip
    if corr2pwr > Self::CORR_TO_PWR_MIN.max(self.peak_value) && cosine_sim > Self::COSINE_MIN {
      self.peak_value = corr2pwr;
      self.peak_index = self.index;
      FramingState::DetectPreamble
    } else if self.index - self.peak_index > Self::AFTER_PEAK_SAMPLES && self.peak_index != 0 {
      self.detect_window.iter_mut().for_each(|x| *x = 0.0);
      self.detect_sqr_sum = 0.0;

      let frame_samples = self.stream_range(self.peak_index + 1, self.index);
      self.frame_window.extend(frame_samples);

      // reset correlation max and argmax
      self.peak_value = 0.0;
      self.peak_index = 0;

      FramingState::WaitPayload
    } else {
      FramingState::DetectPreamble
    }
  }
  fn iter_wait_frame(&mut self, sample: f32) -> (FramingState, Option<Frame>) {
    assert!(self.frame_window.len() < self.payload_len);
    self.frame_window.push_back(sample);
    // wait until we have enough example
    if self.frame_window.len() == self.payload_len {
      let frame = self.frame_window.drain(..).collect();
      (FramingState::DetectPreamble, Some(frame))
    } else {
      (FramingState::WaitPayload, None)
    }
  }
}

impl FrameDetector for CorrelationFraming {
  fn update(&mut self, sample: f32) -> Option<Frame> {
    // moving average smooth filtering
    self.power = self.power * 63.0 / 64.0 + sample * sample / 64.0;
    // update the stream buffer
    self.index += 1;
    self.stream_window.push_back(sample);
    if self.stream_window.len() > self.payload_len * 2 {
      self.stream_window.pop_front();
      self.stream_head += 1;
    }

    // state transition
    match self.state {
      FramingState::DetectPreamble => {
        self.state = self.iter_detect_preamble(sample);
        None
      }
      FramingState::WaitPayload => {
        let (st, frame) = self.iter_wait_frame(sample);
        self.state = st;
        // the possible found frame
        frame
      }
    }
  }

  fn new(preamble_sequence: Vec<f32>, payload_samples: usize) -> Self {
    let m = preamble_sequence.len();
    let n = m + payload_samples;
    let sqr_sum = preamble_sequence.iter().fold(0.0, |s, &x| s + x * x);
    Self {
      preamble_len: m,
      payload_len: payload_samples,
      preamble: preamble_sequence,
      preamble_norm: sqr_sum.sqrt(),
      state: FramingState::DetectPreamble,
      index: 0,
      stream_head: 0,
      power: sqr_sum / m as f32,
      peak_value: 0.0,
      peak_index: 0,
      detect_window: std::iter::repeat(0.0).take(m).collect(),
      detect_sqr_sum: 0.0,
      frame_window: VecDeque::with_capacity(n),
      stream_window: VecDeque::with_capacity(n * 2 + 1),
    }
  }
}

#[cfg(test)]
mod tests {

  use crate::phy_packet::{preambles::ChirpUpDown, FrameDetector, PreambleGenerator};

  use super::CorrelationFraming;

  #[test]
  fn corr_detect() {
    const PL_LEN: usize = 500;

    let preamble: Vec<_> = ChirpUpDown::generate_preamble().collect();
    let mut detector = CorrelationFraming::new(preamble.clone(), PL_LEN);

    // preamble sequence
    preamble
      .iter()
      .map(|x| x * 0.8 + 0.1 * x.sin())
      .for_each(|x| assert_eq!(detector.update(x), None));

    // the payload: exactly one frame will be found
    let mut s = 0;
    for i in 0..PL_LEN * 2 {
      let v = (0.33 * i as f32).sin();
      if detector.update(v).is_some() {
        println!("detected {}/{}", i, PL_LEN);
        s += 1;
      }
    }
    assert_eq!(s, 1);

    // trash: no frame will be found
    s = 0;
    for i in 0..PL_LEN * 2 {
      let v = (0.33 * i as f32).sin();
      if detector.update(v).is_some() {
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

    let preamble: Vec<_> = ChirpUpDown::generate_preamble().collect();
    let mut detector = CorrelationFraming::new(preamble.clone(), PL_LEN);

    // trash sequence
    let recv: Vec<_> = (0..PRE_LEN).map(|x| x as f32).collect();
    recv.iter().for_each(|&x| assert_eq!(detector.update(x), None));

    // more random stuff
    let mut s = 0;
    for i in 0..PL_LEN * 20 {
      let v = (0.33 * i as f32).sin();
      if detector.update(v).is_some() {
        println!("detected {}/{}", i, PL_LEN);
        s += 1;
      }
    }
    // expectation: no frame found
    assert_eq!(s, 0);
  }
}

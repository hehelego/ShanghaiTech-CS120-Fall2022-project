use super::{Frame, FrameDetector};
use crate::helper::dot_product;
use std::collections::VecDeque;

enum FramingState {
  DetectPreamble,
  WaitPayload,
}

pub struct CorrelationFraming {
  preamble_len: usize,
  payload_len: usize,
  preamble: Vec<f32>,
  state: FramingState,
  index: usize,
  stream_head: usize,
  power: f32,
  peak_value: f32,
  peak_index: usize,
  detect_window: VecDeque<f32>,
  frame_window: VecDeque<f32>,
  stream_window: VecDeque<f32>,
}

impl CorrelationFraming {
  pub const CORR_PWR_RATIO_THRESHOLD: f32 = 1.5;
  pub const PWR_MIN: f32 = 0.05;
  pub const AFTER_PEAK_SAMPLES: usize = 32;

  fn stream_range(&self, start: usize, end: usize) -> Vec<f32> {
    let offset = self.stream_head;
    let start = start - offset;
    let end = end - offset;
    self.stream_window.range(start..end).cloned().collect()
  }
  // stop when the Cosine-Similarity of detect window and the preamble is greater than a threshold
  fn iter_detect_preamble(&mut self, sample: f32) -> FramingState {
    // update the preamble detect window
    self.detect_window.pop_front();
    self.detect_window.push_back(sample);

    // compute the correlation
    let corr = 2.0 * dot_product(self.detect_window.iter(), self.preamble.iter()) / self.preamble_len as f32;

    // - max correlation: end of the preamble sequence
    // - correlation drops down: payload section begins
    // - otherwise: skip
    if corr > self.power * Self::CORR_PWR_RATIO_THRESHOLD && corr > self.peak_value.max(Self::PWR_MIN) {
      self.peak_value = corr;
      self.peak_index = self.index;
      FramingState::DetectPreamble
    } else if self.index - self.peak_index > Self::AFTER_PEAK_SAMPLES && self.peak_index != 0 {
      self.detect_window.iter_mut().for_each(|x| *x = 0.0);
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
    Self {
      preamble_len: m,
      payload_len: payload_samples,
      preamble: preamble_sequence,
      state: FramingState::DetectPreamble,
      index: 0,
      stream_head: 0,
      power: 0.0,
      peak_value: 0.0,
      peak_index: 0,
      detect_window: std::iter::repeat(0.0).take(m).collect(),
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

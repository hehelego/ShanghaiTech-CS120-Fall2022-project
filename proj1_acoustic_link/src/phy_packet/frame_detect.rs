use super::{Frame, FrameDetector};
use crate::helper::dot_product;
use std::collections::VecDeque;

enum FramingState {
  DetectPreamble,
  WaitPayload,
}

/// Correlation Preamble Detection Algorithm: correlation sequence falling edge.  
/// credit: [Open OFDM docs: detection](https://openofdm.readthedocs.io/en/latest/detection.html).  
/// See also: `SamplePHY.m` provided by Prof. Zhice Yang.
///
/// If a preamble exists, the correlation pattern is:
/// 1. first goes up quickly when the preamble starts
/// 2. stay at plateau for a while
/// 3. reduce quickly after the preamble ends
/// Our approach is to detect **the falling edge**
///
/// Another necessary condition is that the waveform
/// bears enough similarity with the preamble sequence.  
/// We test whether the cosine-similarity is greater than [`CorrelationFraming::COSINE_MIN`]
pub struct CorrelationFraming {
  preamble_len: usize,           // number of samples in the preamble sequence
  payload_len: usize,            // number of samples in the payload section
  preamble: Vec<f32>,            // preamble sample sequence
  preamble_norm: f32,            // norm of the preamble vector: square root of the square sum of preamble samples
  state: FramingState,           // current working state of the frame detector: either
  index: usize,                  // the index of the incomming sample: 1-based indexing
  stream_head: usize,            // the index of the first sample in the stream buffer
  power: f32,                    // smoothed input signal power
  peak_value: f32,               // maximum value of the correlation
  peak_index: usize,             // the sample index when the correlation peak value is found
  detect_window: VecDeque<f32>,  // a sliding window containg the possible preamble section.
  detect_sqr_sum: f32,           // the square sum of the samples in the detect window
  payload_window: VecDeque<f32>, // the samples in the payload section of a frame.
  // used only when a preamble is detected.
  stream_window: VecDeque<f32>, // the stream buffer, used to preserve the input sample sequence
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

  /// Extract the samples whose index falls in [start, end-1], pack then into a vector.
  fn stream_range(&self, start: usize, end: usize) -> Vec<f32> {
    // the sample on the stream buffer head has index `self.stream_head`
    let offset = self.stream_head;
    // compute the range in the stream buffer
    let start = start - offset;
    let end = end - offset;
    // clone the samples in the range
    self.stream_window.range(start..end).cloned().collect()
  }

  /// The actions and state-transitions to do when the frame detector state is `DetectPreamble`.
  fn iter_detect_preamble(&mut self, sample: f32) -> FramingState {
    // update the preamble detect window
    let head = self.detect_window.pop_front().unwrap();
    self.detect_window.push_back(sample);
    self.detect_sqr_sum += sample * sample - head * head;

    // compute the dot product, correlation and cosine-similarity
    let dot = dot_product(self.detect_window.iter(), self.preamble.iter());
    let corr2pwr = (dot / self.preamble_len as f32) / self.power;
    let cosine_sim = dot / self.preamble_norm / self.detect_sqr_sum.sqrt();

    // the three possible branches:
    // - max correlation index: end of the preamble sequence
    // - correlation falling edge: preamble ends and payload begins
    // - otherwise: skip
    if corr2pwr > Self::CORR_TO_PWR_MIN.max(self.peak_value) && cosine_sim > Self::COSINE_MIN {
      // entering the correlation plateau phase

      self.peak_value = corr2pwr;
      self.peak_index = self.index;
      // next: wait for falling edge
      FramingState::DetectPreamble
    } else if self.index - self.peak_index > Self::AFTER_PEAK_SAMPLES && self.peak_index != 0 {
      // find the correlation falling edge, a preamble is detected

      // clear the preamble detect window
      self.detect_window.iter_mut().for_each(|x| *x = 0.0);
      self.detect_sqr_sum = 0.0;

      // samples after the correlation peak and the correlation falling edge
      // are the beginning samples of the payload section.
      // push them into the payload window.
      let frame_samples = self.stream_range(self.peak_index + 1, self.index + 1);
      self.payload_window.extend(frame_samples);

      // reset correlation peak value and peak index
      self.peak_value = 0.0;
      self.peak_index = 0;

      // the preamble is found.
      // next: wait for the payload section.
      FramingState::WaitPayload
    } else {
      // otherwise, try to detect the preamble at the next position
      FramingState::DetectPreamble
    }
  }
  /// The actions and state-transitions to do when the frame detector state is `WaitPayload`.
  fn iter_wait_payload(&mut self, sample: f32) -> (FramingState, Option<Frame>) {
    assert!(self.payload_window.len() < self.payload_len);

    // append the newly found sample into the payload window
    self.payload_window.push_back(sample);
    // wait until we have enough example
    if self.payload_window.len() == self.payload_len {
      // extract the payload section, clear the payload window.
      let frame = self.payload_window.drain(..).collect();
      // we have extracted a whole frame,
      // next: try to detect another frame
      (FramingState::DetectPreamble, Some(frame))
    } else {
      // no enough samples, continue to wait for more samples.
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
        let (st, frame) = self.iter_wait_payload(sample);
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
      stream_head: 1,
      power: sqr_sum / m as f32,
      peak_value: 0.0,
      peak_index: 0,
      detect_window: std::iter::repeat(0.0).take(m).collect(),
      detect_sqr_sum: 0.0,
      payload_window: VecDeque::with_capacity(n),
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

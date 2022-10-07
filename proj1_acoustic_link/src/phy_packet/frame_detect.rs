use super::{FrameDetector, FramePayload, PreambleGen};
use crate::helper::dot_product;
use std::collections::VecDeque;

enum FramingState {
  DetectPreamble,
  WaitPayload,
}

struct StreamBuf {
  cap: usize,
  head_index: usize,
  window: VecDeque<f32>,
  smooth_power: f32,
}

impl StreamBuf {
  fn new(cap: usize, init_pwr: f32) -> Self {
    let window = VecDeque::with_capacity(cap);
    Self {
      cap,
      head_index: 1,
      window,
      smooth_power: init_pwr,
    }
  }
  /// Extract the samples whose index falls in [start, end-1].
  fn cloned_range(&self, start: usize, end: usize) -> Vec<f32> {
    let offset = self.head_index;
    let start = start - offset;
    let end = end - offset;
    self.window.range(start..end).cloned().collect()
  }
  fn on_sample(&mut self, sample: f32) {
    self.smooth_power = self.smooth_power * 63.0 / 64.0 + sample * sample / 64.0;
    if self.window.len() == self.cap {
      self.window.pop_front();
      self.head_index += 1;
    }
    self.window.push_back(sample);
  }
}

struct Window {
  cap: usize,
  square_sum: f32,
  window: VecDeque<f32>,
}
impl Window {
  fn new(cap: usize) -> Self {
    let window: VecDeque<_> = vec![0.0; cap].into();
    Self {
      cap,
      square_sum: 0.0,
      window,
    }
  }
  fn iter(&self) -> impl ExactSizeIterator<Item = &'_ f32> {
    self.window.iter()
  }
  fn norm(&self) -> f32 {
    self.square_sum.sqrt()
  }
  fn on_sample(&mut self, sample: f32) {
    if self.window.len() == self.cap {
      let x = self.window.pop_front().unwrap();
      self.square_sum -= x * x;
    }
    self.window.push_back(sample);
    self.square_sum += sample * sample;
  }
  fn clear(&mut self) {
    self.window.iter_mut().for_each(|x| *x = 0.0);
    self.square_sum = 0.0;
  }
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
  // part: preamble
  preamble_len: usize, // number of samples in the preamble sequence
  preamble: Vec<f32>,  // preamble sample sequence
  preamble_norm: f32,  // norm of the preamble vector: square root of the square sum of preamble samples
  // part: state & input
  state: FramingState,    // current working state of the frame detector: either
  stream: StreamBuf,      // a buffer holding the samples comming from the stream
  incomming_index: usize, // the index of the incomming sample: 1-based indexing
  // part: detect state
  detect_peak_val: f32,   // maximum value of the correlation
  detect_peak_idx: usize, // the sample index when the correlation peak value is found
  detect_win: Window,     // a sliding window containg the possible preamble section.
  // part: wait state
  frame_payload: Vec<f32>, // the samples in the payload section of a frame. used only when a preamble is detected.
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

  /// The actions and state-transitions to do when the frame detector state is `DetectPreamble`.
  fn iter_detect_preamble(&mut self, sample: f32) -> FramingState {
    // update the preamble detect window
    self.detect_win.on_sample(sample);

    // compute the dot product, correlation and cosine-similarity
    let dot = dot_product(self.detect_win.iter(), self.preamble.iter());
    let corr2pwr = (dot / self.preamble_len as f32) / self.stream.smooth_power;
    let cosine_sim = dot / self.preamble_norm / self.detect_win.norm();

    // the three possible branches:
    // - max correlation index: end of the preamble sequence
    // - correlation falling edge: preamble ends and payload begins
    // - otherwise: skip
    if corr2pwr > Self::CORR_TO_PWR_MIN.max(self.detect_peak_val) && cosine_sim > Self::COSINE_MIN {
      // entering the correlation plateau phase

      self.detect_peak_val = corr2pwr;
      self.detect_peak_idx = self.incomming_index;
      // next: wait for falling edge
      FramingState::DetectPreamble
    } else if self.incomming_index - self.detect_peak_idx > Self::AFTER_PEAK_SAMPLES && self.detect_peak_idx != 0 {
      // find the correlation falling edge, a preamble is detected

      // clear the preamble detect window
      self.detect_win.clear();

      // samples after the correlation peak and the correlation falling edge
      // are the beginning samples of the payload section.
      // push them into the payload window.
      let frame_samples = self
        .stream
        .cloned_range(self.detect_peak_idx + 1, self.incomming_index + 1);
      self.frame_payload.extend(frame_samples);

      // reset correlation peak value and peak index
      self.detect_peak_val = 0.0;
      self.detect_peak_idx = 0;

      // the preamble is found.
      // next: wait for the payload section.
      FramingState::WaitPayload
    } else {
      // otherwise, try to detect the preamble at the next position
      FramingState::DetectPreamble
    }
  }
  /// The actions and state-transitions to do when the frame detector state is `WaitPayload`.
  fn iter_wait_payload(&mut self, sample: f32) -> (FramingState, Option<FramePayload>) {
    assert!(self.frame_payload.len() < self.frame_payload.capacity());

    // append the newly found sample into the payload window
    self.frame_payload.push(sample);
    // wait until we have enough example
    if self.frame_payload.len() == self.frame_payload.capacity() {
      // extract the payload section, clear the payload window.
      let frame = self.frame_payload.clone();
      self.frame_payload.clear();
      // we have extracted a whole frame,
      // next: try to detect another frame
      (FramingState::DetectPreamble, Some(frame))
    } else {
      // no enough samples, continue to wait for more samples.
      (FramingState::WaitPayload, None)
    }
  }

  /// Create a frame detector.  
  /// To create a frame detector,
  /// the preamble sequence to detect and
  /// the length of the payload section of a frame
  /// should be given.
  pub fn new<PG, const PAYLOAD_LEN: usize>() -> Self
  where
    PG: PreambleGen,
  {
    let frame_len: usize = PG::PREAMBLE_LEN + PAYLOAD_LEN;
    let preamble_sequence = PG::generate();

    let sqr_sum = preamble_sequence.iter().fold(0.0, |s, &x| s + x * x);
    Self {
      // part: preamble
      preamble_len: PG::PREAMBLE_LEN,
      preamble: preamble_sequence,
      preamble_norm: sqr_sum.sqrt(),
      // part: state & input
      state: FramingState::DetectPreamble,
      incomming_index: 0,
      stream: StreamBuf::new(frame_len * 2 + 1, sqr_sum / PG::PREAMBLE_LEN as f32),
      // part: members for detect state
      detect_peak_val: 0.0,
      detect_peak_idx: 0,
      detect_win: Window::new(PG::PREAMBLE_LEN),
      // part: members for wait state
      frame_payload: Vec::with_capacity(PAYLOAD_LEN),
    }
  }
}

impl FrameDetector for CorrelationFraming {
  fn on_sample(&mut self, sample: f32) -> Option<FramePayload> {
    self.incomming_index += 1;
    self.stream.on_sample(sample);

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

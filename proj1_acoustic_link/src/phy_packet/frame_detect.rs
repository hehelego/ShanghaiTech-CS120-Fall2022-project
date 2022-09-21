use std::collections::VecDeque;

use crate::helper::dot_product;

use super::{Frame, FrameDetector};

enum FramingState {
  Detect,
  Decode,
}

/// detecting frames with Cosine-Similarity.  
/// **NOTE:** hard-coded threshold
pub struct CosineSimilarityFraming {
  state: FramingState,
  frame_len: usize,
  preamble: Vec<f32>,
  power: f32,
  detect_window: VecDeque<f32>,
  decode_window: VecDeque<f32>,
}
impl CosineSimilarityFraming {
  // stop when the Cosine-Similarity of detect window and the preamble is greater than a threshold
  fn detect_iter(&mut self, sample: f32) -> FramingState {
    self.detect_window.pop_front();
    self.detect_window.push_back(sample);

    // norm of window vector
    let win_norm = self.detect_window.iter().map(|x| x * x).sum::<f32>().sqrt();
    // norm of preamble vector
    let pre_norm = self.preamble.iter().map(|x| x * x).sum::<f32>().sqrt();
    // the dot product
    let dot = dot_product(self.detect_window.iter(), self.preamble.iter());

    // the Cosine-Similarity
    if dot / (win_norm * pre_norm) > 0.7 {
      // send the preamble sequence to frame queue
      self.decode_window.extend(self.detect_window.iter());
      self.detect_window.iter_mut().for_each(|x| *x = 0.0);
      // wait for a whole frame
      FramingState::Decode
    } else {
      // try to detect preamble on next sample in-comming
      FramingState::Detect
    }
  }
  fn decode_iter(&mut self, sample: f32) -> (FramingState, Option<Frame>) {
    self.decode_window.push_back(sample);
    // wait until we have enough example
    if self.decode_window.len() == self.frame_len {
      let frame = self.decode_window.drain(..).collect();
      (FramingState::Detect, Some(frame))
    } else {
      (FramingState::Decode, None)
    }
  }
}
impl FrameDetector for CosineSimilarityFraming {
  fn update(&mut self, sample: f32) -> Option<Frame> {
    // moving average smooth filtering
    self.power = self.power * 63.0 / 64.0 + sample * sample / 64.0;
    // state transition
    match self.state {
      FramingState::Detect => {
        self.state = self.detect_iter(sample);
        None
      }
      FramingState::Decode => {
        let (st, frame) = self.decode_iter(sample);
        self.state = st;
        // the possible found frame
        frame
      }
    }
  }

  fn new(preamble_samples: Vec<f32>, payload_samples: usize) -> Self {
    let m = preamble_samples.len();
    let n = m + payload_samples;
    Self {
      state: FramingState::Detect,
      frame_len: n,
      preamble: preamble_samples,
      power: 0.0,
      detect_window: std::iter::repeat(0.0).take(m).collect(),
      decode_window: VecDeque::with_capacity(n),
    }
  }
}

#[cfg(test)]
mod tests {

  use crate::phy_packet::{preambles::ChirpUpDown, FrameDetector, PreambleGenerator};

  use super::CosineSimilarityFraming;

  #[test]
  fn cosine_detect() {
    const PL_LEN: usize = 10;

    let preamble: Vec<_> = ChirpUpDown::generate_preamble().collect();
    let mut detector = CosineSimilarityFraming::new(preamble.clone(), PL_LEN);
    preamble.iter().for_each(|&x| assert_eq!(detector.update(x), None));

    let mut s = 0;
    for i in 0..PL_LEN * 2 {
      let v = 0.33 * i as f32;
      if detector.update(v).is_some() {
        println!("detected {}/{}", i, PL_LEN);
        s += 1;
      }
    }
    assert_eq!(s, 1);
  }

  #[test]
  fn cosine_detect_none() {
    const PRE_LEN: usize = 200;
    const PL_LEN: usize = 10;

    let preamble: Vec<_> = ChirpUpDown::generate_preamble().collect();
    let recv: Vec<_> = (0..PRE_LEN).map(|x| x as f32).collect();
    let mut detector = CosineSimilarityFraming::new(preamble.clone(), PL_LEN);
    recv.iter().for_each(|&x| assert_eq!(detector.update(x), None));

    let mut s = 0;
    for i in 0..PL_LEN * 2 {
      let v = 0.33 * i as f32;
      if detector.update(v).is_some() {
        println!("detected {}/{}", i, PL_LEN);
        s += 1;
      }
    }
    assert_eq!(s, 0);
  }
}

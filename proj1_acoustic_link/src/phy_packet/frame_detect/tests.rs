#[cfg(feature = "wired")]
mod wired_tests {
  use crate::phy_packet::{frame_detect::CorrelationFraming, preambles::ChirpUpDown, FrameDetector, PreambleGen};
  use crate::sample_stream::{HoundInStream, HoundOutStream};
  use crate::traits::{InStream, OutStream, Sample, FP};

  #[test]
  fn corr_detect() {
    const PL_LEN: usize = 500;

    let preamble = ChirpUpDown::generate();
    let mut detector = CorrelationFraming::new::<PL_LEN>(ChirpUpDown::new());

    // preamble sequence
    preamble
      .iter()
      .map(|x| x * FP::from_f32(0.8) + FP::from_f32(0.1) * x.sin())
      .for_each(|x| assert_eq!(detector.on_sample(x), None));

    // the payload: exactly one frame will be found
    let mut s = 0;
    for i in 0..PL_LEN * 2 {
      let v = FP::from_f32(0.33 * i as f32).sin();
      if detector.on_sample(v).is_some() {
        println!("detected {}/{}", i, PL_LEN);
        s += 1;
      }
    }
    assert_eq!(s, 1);

    // trash: no frame will be found
    s = 0;
    for i in 0..PL_LEN * 2 {
      let v = FP::from_f32(0.33 * i as f32).sin();
      if detector.on_sample(v).is_some() {
        println!("detected {}/{}", i, PL_LEN);
        s += 1;
      }
    }
    assert_eq!(s, 0);
  }
  #[test]
  fn corr_detect_multi() {
    const PL_LEN: usize = 500;
    const N: usize = 10;

    let preamble = ChirpUpDown::generate();
    let mut detector = CorrelationFraming::new::<PL_LEN>(ChirpUpDown::new());
    let mut s = 0;

    for j in 1..N {
      // preamble sequence
      preamble
        .iter()
        .map(|x| x * FP::from_f32(0.8) + FP::from_f32(0.1) * x.sin())
        .for_each(|x| assert_eq!(detector.on_sample(x), None));

      // the payload: exactly one frame will be found
      for i in 0..PL_LEN * 2 {
      let v = FP::from_f32(0.33 * i as f32).sin();
        if detector.on_sample(v).is_some() {
          println!("detected {}/{}", i, PL_LEN);
          s += 1;
        }
      }
      assert_eq!(s, j);
    }

    // trash: no frame will be found
    s = 0;
    for i in 0..PL_LEN * 2 {
      let v = FP::from_f32(0.33 * i as f32).sin();
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

    let mut detector = CorrelationFraming::new::<PL_LEN>(ChirpUpDown::new());

    // trash sequence
    let recv: Vec<_> = (0..PRE_LEN).map(|x| FP::from_f32(x as f32)).collect();
    recv.iter().for_each(|&x| assert_eq!(detector.on_sample(x), None));

    // more random stuff
    let mut s = 0;
    for i in 0..PL_LEN * 20 {
      let v = FP::from_f32(0.33 * i as f32).sin();
      if detector.on_sample(v).is_some() {
        println!("detected {}/{}", i, PL_LEN);
        s += 1;
      }
    }
    // expectation: no frame found
    assert_eq!(s, 0);
  }

  #[test]
  fn corr_detect_air_gen() {
    const PL_LEN: usize = 500;
    const FILE_NAME: &str = "corr_detect_air.wav";
    // Write samples to the file.
    let mut hound_out_stream = HoundOutStream::create(FILE_NAME);
    hound_out_stream
      .write(ChirpUpDown::generate().samples().as_slice())
      .unwrap();
    let payload: Vec<FP> = (0..PL_LEN * 2).map(|x| FP::from_f32(x as f32 * 0.33).sin()).collect();
    hound_out_stream.write(payload.as_slice()).unwrap();
    hound_out_stream.finalize();
    // Read the samples and try to detect samples again.
    let mut hound_in_stream = HoundInStream::open(FILE_NAME);
    let mut buf = [FP::ZERO; ChirpUpDown::N + 2 * PL_LEN];
    let mut detector = CorrelationFraming::new::<PL_LEN>(ChirpUpDown::new());
    let mut s = 0;
    hound_in_stream.read_exact(&mut buf).unwrap();
    for x in buf.iter() {
      if let Some(payload_recv) = detector.on_sample(*x) {
        assert_eq!(&payload_recv, &payload[..PL_LEN]);
        s += 1;
      }
    }
    assert_eq!(s, 1);
  }

  #[test]
  fn corr_detect_air_multi_gen() {
    const PL_LEN: usize = 2500;
    const FILE_NAME: &str = "corr_detect_air_multi.wav";
    const PACKET_NUM: usize = 20;
    // Write samples to the file.
    let mut hound_out_stream = HoundOutStream::create(FILE_NAME);
    let payload: Vec<FP> = (0..PL_LEN).map(|x| FP::from_f32(x as f32 * 0.33).sin()).collect();

    for _ in 0..PACKET_NUM {
      hound_out_stream
        .write(ChirpUpDown::generate().samples().as_slice())
        .unwrap();
      hound_out_stream.write(payload.as_slice()).unwrap();
    }
    hound_out_stream.finalize();
    // Read the samples and try to detect samples again.
    let mut hound_in_stream = HoundInStream::open(FILE_NAME);
    let mut buf = [FP::ZERO; ChirpUpDown::N + PL_LEN];
    let mut detector = CorrelationFraming::new::<PL_LEN>(ChirpUpDown::new());
    let mut s = 0;
    while let Ok(num) = hound_in_stream.read(&mut buf) {
      if num == 0 {
        break;
      }
      for x in buf.iter() {
        if let Some(payload_recv) = detector.on_sample(*x) {
          assert_eq!(&payload_recv, &payload[..PL_LEN]);
          s += 1;
        }
      }
    }
    assert_eq!(s, PACKET_NUM);
  }
}

#[cfg(not(feature = "wired"))]
mod wireless_tests {
  use crate::phy_packet::{frame_detect::CorrelationFraming, preambles::ChirpUpDown, FrameDetector};
  use crate::sample_stream::CpalInStream;
  use crate::traits::InStream;
  use crate::traits::FP;

  #[test]
  fn corr_detect_air_recv() {
    const PL_LEN: usize = 500;
    let mut buf = [FP::ZERO; ChirpUpDown::N + PL_LEN];
    let mut cpal_in_stream = CpalInStream::default();
    let mut s = 0;
    let mut detector = CorrelationFraming::new::<PL_LEN>(ChirpUpDown::new());

    'outer: loop {
      cpal_in_stream.read_exact(&mut buf).unwrap();
      for x in buf.iter() {
        if detector.on_sample(*x).is_some() {
          s += 1;
          break 'outer;
        }
      }
    }
    assert_eq!(s, 1);
  }

  #[test]
  fn corr_detect_air_multi_recv() {
    use std::time::Duration;

    const PL_LEN: usize = 2000;
    const PACKET_NUM: usize = 20;
    let mut buf = [FP::ZERO; ChirpUpDown::N + PL_LEN];
    let mut cpal_in_stream = CpalInStream::default();
    let mut s = 0;
    let mut detector = CorrelationFraming::new::<PL_LEN>(ChirpUpDown::new());

    let mut last_time = std::time::Instant::now();
    'outer: loop {
      let n = cpal_in_stream.read(&mut buf).unwrap();
      for x in buf.iter().take(n) {
        if detector.on_sample(*x).is_some() {
          last_time = std::time::Instant::now();
          s += 1;
        }
      }
      if last_time.elapsed() > Duration::from_secs(3) {
        break 'outer;
      }
    }
    assert_eq!(s, PACKET_NUM);
  }
}

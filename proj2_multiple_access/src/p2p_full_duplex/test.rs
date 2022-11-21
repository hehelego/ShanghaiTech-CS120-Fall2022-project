use std::time::Duration;

#[test]
fn packet_send() {
  use crate::{MacAddr, MacLayer, MacPacket};
  use proj1_acoustic_link::phy_layer::DefaultPhy;
  use std::{fs, thread, time::Instant};

  const INPUT_FILES: &str = "INPUT.bin";
  const SEND_BYTES: usize = 6250 / 8;

  println!("Sending packet from 1 to 2");
  let start = Instant::now();
  let data = fs::read(INPUT_FILES).unwrap();
  let mut mac_layer = MacLayer::new_with_default_phy(MacAddr(1));
  for bytes in data.chunks_exact(MacPacket::<DefaultPhy>::PAYLOAD_SIZE) {
    mac_layer.send_to(MacAddr(2), bytes.to_vec())
  }
  thread::sleep(Duration::from_secs(15));
  println!("Sending {SEND_BYTES} in {}s", (Instant::now() - start).as_secs());
}

#[test]
fn packet_recv() {
  use crate::{MacAddr, MacLayer};
  use std::{fs, fs::File, io::Write, time::Duration};

  const OUTPUT_FILES: &str = "OUTPUT.bin";
  const RECV_BYTES: usize = 6250 / 8;

  println!("receving data...");
  let mut mac_layer = MacLayer::new_with_default_phy(MacAddr(2));
  let mut data = Vec::new();
  let mut total_bytes = 0;
  while let Some(datum) = mac_layer.recv_timeout(Duration::from_secs(3)) {
    total_bytes += datum.len();
    data.extend(datum);
  }
  fs::write(OUTPUT_FILES, &data).unwrap();
  println!("receive {total_bytes}/{RECV_BYTES}");
}

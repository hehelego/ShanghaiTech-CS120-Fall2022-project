use crate::{MacAddr, MacLayer};
use proj1_acoustic_link::helper::{add_padding, remove_padding};
use std::{
  fs, thread,
  time::{Duration, Instant},
};

const INPUT_FILES: &str = "INPUT.bin";
const OUTPUT_FILES: &str = "OUTPUT.bin";
const FILE_SIZE: usize = 6250;

#[test]
fn packet_send() {
  println!("Sending packet from 1 to 2");
  // read input data from file
  let mut data = fs::read(INPUT_FILES).unwrap();
  add_padding(&mut data, 0, MacLayer::MTU);
  // transmission
  let mut mac_layer = MacLayer::new_with_default_phy(MacAddr(1));
  for bytes in data.chunks_exact(MacLayer::MTU) {
    mac_layer.send_to(MacAddr(2), bytes.to_vec())
  }
  // wait for transmission to finish
  thread::sleep(Duration::from_secs(15));
}

#[test]
fn packet_recv() {
  // start timing
  let start = Instant::now();
  println!("receving data...");
  // receive data packet from peer
  let mut mac_layer = MacLayer::new_with_default_phy(MacAddr(2));
  let mut data = vec![];
  while let Some(packet) = mac_layer.recv_timeout(Duration::from_secs(3)) {
    data.extend(packet);
  }
  // stop timing
  println!("finished in {}ms", start.elapsed().as_millis());
  // write result
  remove_padding(&mut data, FILE_SIZE, MacLayer::MTU);
  fs::write(OUTPUT_FILES, &data).unwrap();
}

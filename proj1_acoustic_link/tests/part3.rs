use proj1_acoustic_link::{
  helper::*,
  phy_layer::{PhyTrait, PlainPHY},
  traits::{PacketReceiver, PacketSender},
};
use std::fs;
use std::thread::sleep;
use std::time::Duration;

const CHUNK_LEN: usize = PlainPHY::PACKET_BYTES;
const DATA_LEN: usize = 10000 / 8;
const PAD_LEN: usize = (CHUNK_LEN - DATA_LEN % CHUNK_LEN) % CHUNK_LEN;

#[test]
#[ignore]
fn part3_ck1_send() {
  const FILEPATH: &str = "INPUT.txt";
  let mut physics_layer = PlainPHY::default();

  let data_string = fs::read_to_string(FILEPATH).unwrap();
  let bits = chars_to_bits(data_string.trim_end());

  let mut bytes = bits_to_bytes(&bits);
  add_padding(&mut bytes, 0, CHUNK_LEN);

  println!("send {} packets", bytes.len() / CHUNK_LEN);
  bytes.chunks_exact(CHUNK_LEN).for_each(|chunk| {
    physics_layer.send(chunk.into()).unwrap();
  });

  sleep(Duration::from_secs(15));
}

#[test]
#[ignore]
fn part3_ck1_recv() {
  const FILEPATH: &str = "OUTPUT.txt";
  let mut physics_layer = PlainPHY::default();

  let mut bytes = vec![0; DATA_LEN + PAD_LEN];
  bytes.chunks_exact_mut(CHUNK_LEN).enumerate().for_each(|(idx, chunk)| {
    let packet = physics_layer.recv_timeout(Duration::from_secs(1)).unwrap();
    chunk.copy_from_slice(&packet);
    println!("recv [{}/{}]", idx + 1, (DATA_LEN + PAD_LEN) / CHUNK_LEN);
  });
  remove_padding(&mut bytes, DATA_LEN, CHUNK_LEN);

  let bits = bytes_to_bits(&bytes);
  let data_string = bits_to_chars(&bits);
  fs::write(FILEPATH, data_string).unwrap();
}

use crc::{Crc, CRC_64_ECMA_182};
use proj1_acoustic_link::{
  helper::{bytes_to_bits, bits_to_bytes},
  phy_layer::{ecc_recv, ecc_send},
};
use rand::{distributions::Standard, Rng};

fn checksum(bytes: &[u8]) -> u64 {
  let crc = Crc::<u64>::new(&CRC_64_ECMA_182);
  crc.checksum(bytes)
}

#[test]
#[ignore]
pub fn part4_send() {
  let bytes: Vec<u8> = rand::thread_rng().sample_iter(Standard).take(10000 / 8).collect();
  println!("send {:?}", checksum(&bytes));
  ecc_send(&bytes);
}

#[test]
#[ignore]
pub fn part4_recv() {
  let mut bytes: Vec<u8> = vec![0; 10000 / 8];
  ecc_recv(&mut bytes);
  println!("recv {:?}", checksum(&bytes));
}

#[test]
#[ignore]
pub fn part4_sendfile() {
  let bits: Vec<u8> = std::fs::read_to_string("input.txt")
    .unwrap()
    .trim_end()
    .chars()
    .map(|c| match c {
      '0' => 0,
      '1' => 1,
      _ => panic!("unexpected char in input"),
    })
    .collect();
  assert_eq!(bits.len(), 10000);
  let bytes = bits_to_bytes(&bits);
  println!("send {:?}", checksum(&bytes));
  ecc_send(&bytes);
}

#[test]
#[ignore]
pub fn part4_recvfile() {
  let mut bytes: Vec<u8> = vec![0; 10000 / 8];
  ecc_recv(&mut bytes);
  println!("recv {:?}", checksum(&bytes));
  let bits = bytes_to_bits(&bytes);
  let bits: String = bits
    .iter()
    .map(|b| match b {
      0 => '0',
      1 => '1',
      _ => panic!("unexpected bit"),
    })
    .collect();
  std::fs::write("output.txt", bits).unwrap();
}

use crc::{Crc, CRC_64_ECMA_182};
use proj1_acoustic_link::phy_layer::{ecc_recv, ecc_send};
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

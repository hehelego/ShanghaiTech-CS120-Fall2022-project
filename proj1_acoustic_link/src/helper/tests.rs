use bitvec::prelude::*;
use rand::{distributions::Standard, Rng, RngCore};

use super::{add_padding, decode_4b5b, decode_nrzi, encode_4b5b, encode_nrzi, remove_padding, CrcSeq};

type CS = CrcSeq<9>;
const TESTS: usize = 100;

#[test]
fn pad_add_remove() {
  for _ in 0..TESTS {
    let mut rng = rand::thread_rng();
    let data_len: usize = rng.gen_range(1000..2000);
    let chunk_len: usize = rng.gen_range(1..10);

    let pad_value: u8 = rng.gen();
    let data: Vec<u8> = rng.sample_iter(rand::distributions::Standard).take(data_len).collect();
    let mut data_pad = data.clone();
    add_padding(&mut data_pad, pad_value, chunk_len);
    remove_padding(&mut data_pad, data_len, chunk_len);
    assert_eq!(data_pad, data)
  }
}

fn gen_pack() -> Vec<u8> {
  let mut rng = rand::thread_rng();
  let mut data = vec![0; CS::DATA_SIZE];
  rng.fill_bytes(&mut data);
  let seq = rng.gen_range(0..4);
  CS::pack(&data, seq)
}
fn flip_bit(data: &mut [u8]) {
  let mut rng = rand::thread_rng();
  let byte = rng.gen_range(0..data.len());
  let bit = rng.gen_range(0..8);
  data[byte] ^= 1 << bit;
}

#[test]
fn crcseq_ok() {
  for _ in 0..TESTS {
    let pack = gen_pack();
    if let Some((data, seq)) = CS::unpack(&pack) {
      assert_eq!(CS::pack(&data, seq), pack)
    } else {
      panic!("add-remove identity not hold failed");
    }
  }
}
#[test]
fn crcseq_err() {
  for _ in 0..TESTS {
    let mut pack = gen_pack();
    flip_bit(&mut pack);
    assert_eq!(CS::unpack(&pack), None);
  }
}

#[test]
fn fbfb() {
  for _ in 0..TESTS {
    let bytes: Vec<u8> = rand::thread_rng().sample_iter(Standard).take(1).collect();
    let bits = bytes.view_bits();
    let enc = encode_4b5b(bits.to_owned());
    let dec = decode_4b5b(enc);
    assert_eq!(dec, bits);
  }
}

#[test]
fn nrzi(){
  for _ in 0..TESTS {
    let bytes: Vec<u8> = rand::thread_rng().sample_iter(Standard).take(1).collect();
    let bits = bytes.view_bits();
    let enc = encode_nrzi(bits.to_owned());
    let dec = decode_nrzi(enc);
    assert_eq!(dec, bits);
  }
}

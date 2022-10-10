use proj1_acoustic_link::helper::{bits_to_bytes, bytes_to_bits};
use proj1_acoustic_link::phy_layer::PlainPHY;
use proj1_acoustic_link::traits::{PacketReceiver, PacketSender};
use rand::Rng;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

/// add padding at the beginning of the sequence,
/// the padding section are filled with `pad_value`.
/// make sure that `seq.len() % chunk_len == 0`
fn add_padding<T: Clone>(seq: &mut Vec<T>, pad_value: T, chunk_len: usize) {
  let padding_len = (chunk_len - seq.len() % chunk_len) % chunk_len;
  seq.extend(std::iter::repeat(pad_value).take(padding_len));
}
/// remove the padding section in front of the original data.
/// the length of the original sequence and the padding chunk len should be given.
fn remove_padding<T: Clone>(seq: &mut Vec<T>, original_len: usize, chunk_len: usize) {
  let _ = chunk_len;
  seq.resize(original_len, seq[0].clone());
}

#[test]
fn pad_add_remove() {
  const TESTS: usize = 20;
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

const CHUNK_LEN: usize = PlainPHY::PACKET_BYTES;
const DATA_LEN: usize = 10000 / 8;
const PAD_LEN: usize = (CHUNK_LEN - DATA_LEN % CHUNK_LEN) % CHUNK_LEN;

#[test]
#[ignore]
fn part3_ck1_send() {
  const FILEPATH: &str = "INPUT.txt";
  let mut physics_layer = PlainPHY::default();

  let data_string = fs::read_to_string(FILEPATH).unwrap();
  let bits: Vec<_> = data_string
    .trim_end()
    .chars()
    .map(|x| match x {
      '0' => 0,
      '1' => 1,
      _ => panic!("not a 0/1 bit"),
    })
    .collect();

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
    let packet = physics_layer.recv().unwrap();
    chunk.copy_from_slice(&packet);
    println!("recv [{}/{}]", idx + 1, (DATA_LEN + PAD_LEN) / CHUNK_LEN);
  });
  remove_padding(&mut bytes, DATA_LEN, CHUNK_LEN);

  let bits = bytes_to_bits(&bytes);
  let data_string: String = bits
    .iter()
    .map(|x| match x {
      0 => '0',
      1 => '1',
      _ => panic!("not a 0/1 bit"),
    })
    .collect();
  fs::write(FILEPATH, data_string).unwrap();
}
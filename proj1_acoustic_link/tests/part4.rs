use proj1_acoustic_link::{
  helper::*,
  phy_layer::AtomicPHY,
  phy_layer::PacketError,
  phy_packet::PhyPacket,
  traits::{PacketReceiver, PacketSender},
};
use reed_solomon_erasure::{galois_8::Field, ReedSolomon};
use std::fs::{read_to_string, write as write_string};
use std::thread::sleep;
use std::time::Duration;

const CHUNK_LEN: usize = AtomicPHY::PACKET_BYTES;
const DATA_LEN: usize = 10000 / 8;

const ECC_CHUNKS: usize = 30;
const DATA_CHUNKS: usize = DATA_LEN / CHUNK_LEN + if DATA_LEN % CHUNK_LEN == 0 { 0 } else { 1 };
const TOTAL_CHUNKS: usize = DATA_CHUNKS + ECC_CHUNKS;

#[test]
#[ignore]
fn part4_send() {
  // read bits, pad data
  const FILE_PATH: &str = "INPUT.txt";
  let data_string = read_to_string(FILE_PATH).unwrap();
  let bits = chars_to_bits(data_string.trim_end());
  let mut bytes = bits_to_bytes(&bits);
  add_padding(&mut bytes, 0, CHUNK_LEN);

  // generate chunks
  let rs = ReedSolomon::<Field>::new(DATA_CHUNKS, ECC_CHUNKS).unwrap();
  let mut chunks: Vec<_> = bytes.chunks_exact(CHUNK_LEN).map(Vec::from).collect();
  chunks.resize(TOTAL_CHUNKS, vec![0; CHUNK_LEN]);
  rs.encode(&mut chunks).unwrap();
  assert_eq!(rs.verify(&chunks), Ok(true));

  // send
  let mut phy = AtomicPHY::default();
  chunks.into_iter().for_each(|chunk| phy.send(chunk.to_vec()).unwrap());

  sleep(Duration::from_secs(30));
}

#[test]
#[ignore]
fn part4_recv() {
  const FILE_PATH: &str = "OUTPUT.txt";

  // recv
  let mut phy = AtomicPHY::default();
  let mut chunks: Vec<Option<Vec<u8>>> = vec![None; TOTAL_CHUNKS];
  let mut cur_chk = 0;
  while cur_chk < chunks.len() {
    match phy.recv_timeout(Duration::from_secs(1)) {
      Err(PacketError::NoPacketAvaiable) => break,
      Ok((packet, skips)) => {
        cur_chk += skips as usize;
        println!("get packet[{}]", cur_chk);
        if cur_chk < chunks.len() {
          chunks[cur_chk] = Some(packet);
        }
        cur_chk += 1;
      }
      Err(PacketError::Lost) => continue,
      Err(PacketError::Corrupt) => continue,
    }
  }

  // reconstruct
  let rs = ReedSolomon::<Field>::new(DATA_CHUNKS, ECC_CHUNKS).unwrap();
  rs.reconstruct(&mut chunks).unwrap();

  // write result
  let bytes: Vec<u8> = chunks
    .into_iter()
    .take(DATA_CHUNKS)
    .flat_map(|chunk| chunk.unwrap())
    .collect();
  let bits = bytes_to_bits(&bytes);
  let data_string = bits_to_chars(&bits);
  std::fs::write(FILE_PATH, data_string).unwrap();
}

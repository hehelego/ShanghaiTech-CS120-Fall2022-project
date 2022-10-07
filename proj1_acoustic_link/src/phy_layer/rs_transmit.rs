use std::{thread, time::Duration};

use crate::{
  helper::copy,
  traits::{PacketReceiver, PacketSender},
  DefaultConfig,
};

use super::CrcPhy;
use reed_solomon_erasure::{galois_8::Field, ReedSolomon};

const SHARD_SIZE: usize = CrcPhy::PACKET_BYTES;
const ZERO_SHARD: [u8; SHARD_SIZE] = [0; SHARD_SIZE];

fn rs_codec(bytes: usize) -> ReedSolomon<Field> {
  let data_shards = bytes / SHARD_SIZE + 1;
  let parity_shards = data_shards * 5 / 16;
  ReedSolomon::new(data_shards, parity_shards).unwrap()
}

pub fn ecc_send(bytes: &[u8]) {
  let mut phy = CrcPhy::default();
  let rs = rs_codec(bytes.len());

  let mut shards = vec![ZERO_SHARD; rs.total_shard_count()];
  bytes.chunks(SHARD_SIZE).zip(shards.iter_mut()).for_each(|(x, y)| {
    copy(y.iter_mut(), x.iter().cloned());
  });

  rs.encode(&mut shards).unwrap();
  assert!(rs.verify(&shards).unwrap());

  for shard in shards {
    phy.send(shard.into()).unwrap();
  }
  let samples = rs.total_shard_count() * CrcPhy::PACKET_SAMPLES;
  let secs = 0.3 + samples as f32 / DefaultConfig::SAMPLE_RATE as f32;
  thread::sleep(Duration::from_secs_f32(secs));
}

pub fn ecc_recv(bytes: &mut [u8]) {
  let mut phy = CrcPhy::default();
  let rs = rs_codec(bytes.len());

  let mut shards = vec![None; rs.total_shard_count()];

  for shard in shards.iter_mut() {
    if let Ok(chunk) = phy.recv() {
      *shard = Some(chunk);
    }
  }

  rs.reconstruct(&mut shards).unwrap();

  bytes.chunks_mut(SHARD_SIZE).zip(shards.into_iter()).for_each(|(x, y)| {
    let y = y.unwrap();
    copy(x.iter_mut(), y.iter().cloned());
  });
}

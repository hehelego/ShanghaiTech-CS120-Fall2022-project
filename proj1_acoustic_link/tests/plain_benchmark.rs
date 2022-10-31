#[cfg(all(feature = "wired", not(feature = "nofloat")))]
#[ignore]
#[test]
fn plain_benchmark() {
  use proj1_acoustic_link::{
    phy_layer::PlainPHY as PHY,
    traits::{PacketReceiver, PacketSender},
  };
  use rand::{distributions::Standard, Rng};
  use std::time::{Duration, Instant};

  // prepare data
  const PACKS: usize = 350;
  const BYTES: usize = PHY::PACKET_BYTES * PACKS;
  let data_send: Vec<u8> = rand::thread_rng().sample_iter(Standard).take(BYTES).collect();
  let mut data_recv: Vec<u8> = vec![0; BYTES];

  dbg!(PHY::PACKET_BYTES);

  // timing
  const RECV_TIMEOUT: Duration = Duration::from_millis(200);
  let start = Instant::now();

  // transmission
  let mut phy = PHY::default();
  for packet_data in data_send.chunks_exact(PHY::PACKET_BYTES) {
    let packet = packet_data.to_vec();
    phy.send(packet).unwrap();
  }
  let mut recv_seq = 0;
  for packet_data in data_recv.chunks_exact_mut(PHY::PACKET_BYTES) {
    let packet = phy.recv_timeout(RECV_TIMEOUT).unwrap();
    packet_data.copy_from_slice(packet.as_slice());
    recv_seq += 1;
    println!("packet {}/{} received", recv_seq, PACKS);
  }

  // checking result and computing bandwidth
  let elapsed = start.elapsed();
  let error_bytes = data_send.iter().zip(data_recv.iter()).filter(|(x, y)| x != y).count();
  println!(
    "{} bytes transmitted in {} seconds, with {} errors",
    BYTES,
    elapsed.as_secs_f32(),
    error_bytes
  );
}

#[cfg(feature = "wired")]
#[ignore]
#[test]
fn plain_benchmark() {
  use proj1_acoustic_link::{
    phy_layer::{CrcPhy as PHY, PhyTrait},
    traits::{PacketReceiver, PacketSender},
  };
  use rand::{distributions::Standard, Rng};
  use std::thread;
  use std::time::{Duration, Instant};

  // prepare data
  const PACK_BYTES: usize = PHY::PACKET_BYTES;
  const SEND_PACKS: usize = 100;
  const TOTAL_BYTES: usize = PACK_BYTES * SEND_PACKS;
  let data: Vec<u8> = rand::thread_rng().sample_iter(Standard).take(TOTAL_BYTES).collect();

  // timing
  const RECV_TIMEOUT: Duration = Duration::from_millis(20);
  const BACKOFF_TIMEOUT: Duration = Duration::from_millis(20);
  let start = Instant::now();

  // transmission
  let mut phy = PHY::default();
  for (idx, packet) in data.chunks_exact(PACK_BYTES).enumerate() {
    let mut retry_before_succ: Option<usize> = None;
    for retry in 0..10 {
      let packet = packet.to_vec();
      phy.send(packet).unwrap();
      if phy.recv_timeout(RECV_TIMEOUT).is_ok() {
        retry_before_succ = Some(retry);
        break;
      } else {
        thread::sleep(BACKOFF_TIMEOUT);
      }
    }
    if let Some(retry) = retry_before_succ {
      println!("packet[{}/{}] received after {} retires", idx, SEND_PACKS, retry);
    } else {
      println!("packet[{}/{}] failed", idx, SEND_PACKS);
    }
  }

  // checking result and computing bandwidth
  let elapsed = start.elapsed();
  println!("{} bytes transmitted in {} seconds", TOTAL_BYTES, elapsed.as_secs_f32(),);
}

use proj1_acoustic_link::{phy_layer::PlainPhy, traits::PacketReceiver, traits::PacketSender};

pub fn test_bytes(i: usize) -> Vec<u8> {
  (0..PlainPhy::PACKET_BYTES)
    .map(|x| {
      let y = x as usize * i;
      let z = (y * (y + 13) + 93) % 256;
      z as u8
    })
    .collect()
}

#[test]
#[ignore]
pub fn plain_send() {
  let mut phy = PlainPhy::default();
  for i in 0..100 {
    let pack: Vec<_> = test_bytes(i);
    println!("send[{}]: {:?}", i, pack);
    phy.send(pack).unwrap();
  }
  std::thread::sleep(std::time::Duration::from_secs(5));
}

#[test]
#[ignore]
pub fn plain_recv() {
  let mut phy = PlainPhy::default();
  for i in 0..100 {
    let pack = phy.recv().unwrap();
    println!("recv[{}]: {:?}", i, pack);
  }
}

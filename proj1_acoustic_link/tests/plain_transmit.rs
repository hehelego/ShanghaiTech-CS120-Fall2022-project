use proj1_acoustic_link::{phy_layer::PlainPhy, traits::PacketReceiver, traits::PacketSender};

#[test]
#[ignore]
pub fn plain_send() {
  let mut phy = PlainPhy::default();
  for i in 0..100 {
    let pack = vec![0; PlainPhy::PACKET_BYTES];
    println!("send[{}]: {:?}", i, pack);
    phy.send(pack).unwrap();
  }
}
pub fn plain_recv() {
  let mut phy = PlainPhy::default();
  for i in 0..100 {
    let pack = phy.recv().unwrap();
    println!("recv[{}]: {:?}", i, pack);
  }
}

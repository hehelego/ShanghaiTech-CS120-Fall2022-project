use crate::{mac::MacStateMachine, MacAddr, MacPacket};
use crossbeam_channel::{Receiver, Sender};
use proj1_acoustic_link::phy_layer::PhyLayer;

/// Simple MAC implementaion for peer to peer full duplex connection:
/// stop-and-wait or sliding window.
pub struct Simple {}

impl Simple {}
impl<PHY: PhyLayer> MacStateMachine<PHY> for Simple {
  fn new(
    phy: PHY,
    self_addr: MacAddr,
    packets_to_send: Receiver<MacPacket<PHY>>,
    packets_received: Sender<MacPacket<PHY>>,
    terminate_signal: Receiver<()>,
  ) -> Self {
    todo!()
  }

  fn run(&mut self) {
    todo!()
  }
}

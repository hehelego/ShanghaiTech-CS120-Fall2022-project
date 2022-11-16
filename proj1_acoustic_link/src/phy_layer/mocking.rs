pub use crate::phy_packet::{Modem, PhyPacket, PreambleGen};
pub use crate::traits::{PacketReceiver, PacketSender};

/// A PHY layer object that allow handling send/recv by custom code
pub trait MockingPhy {
  /// The error type of which send/recv may generates
  type Err;
  /// Start the mocking worker, return the handler.
  fn start() -> Self;
  /// Terminate the worker, should block until the worker exits.
  fn terminate();

  /// Called when [`MockingPhy`] send a packet.
  fn handle_send(&mut self, packet: PhyPacket) -> Result<(), ()>;
  /// Called when [`MockingPhy`] recv a packet.
  fn handle_recv(&mut self) -> Result<PhyPacket, Self::Err>;
  /// Called when [`MockingPhy`] recv a packet.
  fn handle_recv_timeout(&mut self, timeout: std::time::Duration) -> Result<PhyPacket, Self::Err>;
  /// Called when [`MockingPhy`] peek received packet
  fn handle_recv_peek(&mut self) -> bool;
  /// Called when [`MockingPhy`] query the channel power.
  fn handle_energy_get(&mut self) -> f32;
}

impl<W, E> PacketSender<PhyPacket, ()> for W
where
  W: MockingPhy<Err = E>,
{
  fn send(&mut self, packet: PhyPacket) -> Result<(), ()> {
    self.handle_send(packet)
  }
}

impl<W, E> PacketReceiver<PhyPacket, E> for W
where
  W: MockingPhy<Err = E>,
{
  fn recv(&mut self) -> Result<PhyPacket, E> {
    self.handle_recv()
  }

  fn recv_timeout(&mut self, timeout: std::time::Duration) -> Result<PhyPacket, E> {
    self.handle_recv_timeout(timeout)
  }

  fn recv_peek(&mut self) -> bool {
    self.handle_recv_peek()
  }
}

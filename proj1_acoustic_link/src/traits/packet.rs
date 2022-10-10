use std::time::Duration;

/// Send discrete packet of type `T` through the [`PacketSender`].
/// Might encounter error of type `E`
pub trait PacketSender<T, E> {
  /// Send a packet.
  /// The function should return immediately.
  /// Implementor should use a separted thread to push the packet.
  fn send(&mut self, packet: T) -> Result<(), E>;
}
/// Receive discrete packet of type `T` through the [`PacketReceiver`]
/// Might encounter error of type `E`
pub trait PacketReceiver<T, E> {
  /// Receive a packet.
  /// The function should return immediately.
  /// Implementor should use a separted thread to detect packet.
  fn recv(&mut self) -> Result<T, E>;

  /// Receive a packet, retry until error or timeout.
  fn recv_timeout(&mut self, timeout: Duration) -> Result<T, E>;
}

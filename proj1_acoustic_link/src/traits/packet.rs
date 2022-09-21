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
}

/// A TxRx object is capable of reading and writing.
pub trait PacketTxRx<T, E>: PacketSender<T, E> + PacketReceiver<T, E> {}
impl<T, E, TxRx> PacketTxRx<T, E> for TxRx where TxRx: PacketSender<T, E> + PacketReceiver<T, E> {}

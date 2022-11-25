use super::ASockProtocol;

/// A socket for sending/receiving TCP packets.
/// Provides transport layer APIs.
pub struct TcpSocket;

impl TcpSocket {
  pub const PROTOCOL: ASockProtocol = ASockProtocol::TCP;
}

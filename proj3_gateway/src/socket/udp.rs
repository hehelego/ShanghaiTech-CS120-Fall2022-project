use super::ASockProtocol;

/// A socket for sending/receiving UDP packets.
/// Provides transport layer APIs.
pub struct UdpSocket;

impl UdpSocket {
  pub const PROTOCOL: ASockProtocol = ASockProtocol::UDP;
}

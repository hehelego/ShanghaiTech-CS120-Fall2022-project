use super::ASockProtocol;

/// A socket for sending/receiving ICMP packets
pub struct IcmpSocket;

impl IcmpSocket {
  pub const PROTOCOL: ASockProtocol = ASockProtocol::ICMP;
}

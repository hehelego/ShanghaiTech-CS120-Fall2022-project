use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use serde::{Deserialize, Serialize};
use socket2::Protocol;

/// Transport layer protocols for Athernet sockets
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ASockProtocol {
  UDP,
  ICMP,
  TCP,
}
impl Into<Protocol> for ASockProtocol {
  fn into(self) -> Protocol {
    match self {
      ASockProtocol::UDP => Protocol::UDP,
      ASockProtocol::ICMP => Protocol::ICMPV4,
      ASockProtocol::TCP => Protocol::TCP,
    }
  }
}
impl Into<IpNextHeaderProtocol> for ASockProtocol {
  fn into(self) -> IpNextHeaderProtocol {
    match self {
      ASockProtocol::UDP => IpNextHeaderProtocols::Udp,
      ASockProtocol::ICMP => IpNextHeaderProtocols::Icmp,
      ASockProtocol::TCP => IpNextHeaderProtocols::Tcp,
    }
  }
}

impl TryFrom<Protocol> for ASockProtocol {
  type Error = std::io::Error;

  fn try_from(value: socket2::Protocol) -> Result<Self, Self::Error> {
    match value {
      Protocol::ICMPV4 => Ok(ASockProtocol::ICMP),
      Protocol::UDP => Ok(ASockProtocol::UDP),
      Protocol::TCP => Ok(ASockProtocol::TCP),
      _ => Err(std::io::ErrorKind::Unsupported.into()),
    }
  }
}
impl TryFrom<IpNextHeaderProtocol> for ASockProtocol {
  type Error = std::io::Error;

  fn try_from(value: IpNextHeaderProtocol) -> Result<Self, Self::Error> {
    match value {
      IpNextHeaderProtocols::Icmp => Ok(ASockProtocol::ICMP),
      IpNextHeaderProtocols::Udp => Ok(ASockProtocol::UDP),
      IpNextHeaderProtocols::Tcp => Ok(ASockProtocol::TCP),
      _ => Err(std::io::ErrorKind::Unsupported.into()),
    }
  }
}

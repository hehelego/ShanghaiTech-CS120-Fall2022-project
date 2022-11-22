use serde::{Deserialize, Serialize};

/// Transport layer protocols for Athernet sockets
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ASockProtocol {
  UDP,
  ICMP,
  TCP,
}
impl Into<socket2::Protocol> for ASockProtocol {
  fn into(self) -> socket2::Protocol {
    match self {
      ASockProtocol::UDP => socket2::Protocol::UDP,
      ASockProtocol::ICMP => socket2::Protocol::ICMPV4,
      ASockProtocol::TCP => socket2::Protocol::TCP,
    }
  }
}

impl TryFrom<socket2::Protocol> for ASockProtocol {
  type Error = std::io::Error;

  fn try_from(value: socket2::Protocol) -> Result<Self, Self::Error> {
    match value {
      socket2::Protocol::ICMPV4 => Ok(ASockProtocol::ICMP),
      socket2::Protocol::UDP => Ok(ASockProtocol::UDP),
      socket2::Protocol::TCP => Ok(ASockProtocol::TCP),
      _ => Err(std::io::Error::from(std::io::ErrorKind::Unsupported)),
    }
  }
}

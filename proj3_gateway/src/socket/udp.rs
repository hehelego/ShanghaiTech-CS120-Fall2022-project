use super::ASockProtocol;
use crate::aip_layer::IpAccessor;
use pnet::packet::udp::Udp;
use std::{io::Result, net::SocketAddrV4};

/// A socket for sending/receiving UDP packets.
/// Provides transport layer APIs.
pub struct UdpSocket {
  addr: SocketAddrV4,
  accessor: IpAccessor,
}

impl UdpSocket {
  pub const PROTOCOL: ASockProtocol = ASockProtocol::UDP;
  pub const MAX_PAYLOAD_LEN: usize = u16::MAX as usize;
  pub fn bind(addr: SocketAddrV4) -> Result<Self> {
    let path = format!("/tmp/udp_client_{}", addr);
    let accessor = IpAccessor::new(&path)?;
    accessor.bind(Self::PROTOCOL, addr)?;
    Ok(Self { accessor, addr })
  }
  pub fn send_to(&self, buf: &[u8], addr: SocketAddrV4) -> Result<usize> {
    let bytes_written = Self::MAX_PAYLOAD_LEN.min(buf.len());
    let udp_packet = Udp {
      source: self.addr.port(),
      destination: addr.port(),
      length: 8 + bytes_written as u16,
      checksum: 0,
      payload: buf[..bytes_written].to_vec(),
    };
    self.accessor.send_udp(udp_packet, addr)?;
    Ok(bytes_written)
  }
  pub fn recv_from(&self) -> Result<(Vec<u8>, SocketAddrV4)> {
    let (Udp { payload, length, .. }, addr) = self.accessor.recv_udp()?;
    Ok((payload[..(length as usize - 8)].to_vec(), addr))
  }
}

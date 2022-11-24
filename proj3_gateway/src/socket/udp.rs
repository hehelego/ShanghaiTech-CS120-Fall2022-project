use super::ASockProtocol;
use crate::aip_layer::IpAccessor;
use pnet::packet::udp::Udp;
use std::net::SocketAddrV4;

/// A socket for sending/receiving UDP packets.
/// Provides transport layer APIs.
pub struct UdpSocket {
  addr: SocketAddrV4,
  accessor: IpAccessor,
}

impl UdpSocket {
  pub const PROTOCOL: ASockProtocol = ASockProtocol::UDP;
  pub const MAX_PAYLOAD_LEN: usize = u16::MAX as usize;
  pub fn bind(addr: SocketAddrV4) -> std::io::Result<Self> {
    let path = format!("udp_client_{}", addr.to_string());
    let accessor = IpAccessor::new(&path)?;
    accessor.bind(Self::PROTOCOL, addr)?;
    Ok(Self { accessor, addr })
  }
  pub fn send_to(&self, buf: &[u8], addr: SocketAddrV4) -> std::io::Result<usize> {
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
  pub fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddrV4)> {
    let (Udp { payload, length, .. }, addr) = self.accessor.recv_udp()?;
    buf[..length as usize].clone_from_slice(&payload);
    Ok((length as usize, addr))
  }
}

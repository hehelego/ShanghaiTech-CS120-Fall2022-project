use super::ASockProtocol;
use crate::IpAccessor;
use pnet::packet::icmp::{
  echo_reply::EchoReplyPacket,
  echo_request::{EchoRequest, MutableEchoRequestPacket},
  Icmp, IcmpCode, IcmpPacket, IcmpTypes,
};
use pnet::packet::{FromPacket, Packet};
use std::{
  cell::RefCell,
  io::Result,
  net::{Ipv4Addr, SocketAddrV4},
  time::{Duration, Instant},
};

/// A socket for sending/receiving ICMP packets
pub struct IcmpSocket {
  ip_sock: IpAccessor,
  buf: RefCell<Vec<u8>>,
}

impl IcmpSocket {
  pub const PROTOCOL: ASockProtocol = ASockProtocol::ICMP;
  pub fn bind(addr: Ipv4Addr) -> Result<Self> {
    let ipc_path = &(format!("/tmp/icmp_{}", addr));
    let accessor = IpAccessor::new(ipc_path)?;
    accessor.bind(ASockProtocol::ICMP, SocketAddrV4::new(addr, 0))?;
    Ok(Self {
      ip_sock: accessor,
      buf: RefCell::new(Vec::new()),
    })
  }
  pub fn send(&self, icmp: Icmp, dest: Ipv4Addr) -> Result<()> {
    self.ip_sock.send_icmp(icmp, dest)
  }
  pub fn recv(&self) -> Result<(Icmp, Ipv4Addr)> {
    self.ip_sock.recv_icmp()
  }

  /// send an ICMP echo request to `dest`
  pub fn send_ping(&self, id: u16, seq: u16, payload: &[u8], dest: Ipv4Addr) -> Result<()> {
    let mut buf = self.buf.borrow_mut();
    buf.resize(8 + payload.len(), 0);
    let mut pack = MutableEchoRequestPacket::new(&mut buf).unwrap();
    pack.populate(&EchoRequest {
      icmp_type: IcmpTypes::EchoRequest,
      icmp_code: IcmpCode(0),
      checksum: 0,
      identifier: id,
      sequence_number: seq,
      payload: payload.into(),
    });
    let icmp = IcmpPacket::new(pack.packet()).unwrap().from_packet();

    self.send(icmp, dest)
  }
  /// wait for an ICMP echo response from `from`
  pub fn recv_pong_timeout(&self, id: u16, seq: u16, from: Ipv4Addr, duration: Duration) -> Result<Vec<u8>> {
    let start = Instant::now();
    while start.elapsed() < duration {
      if let Ok((icmp, src)) = self.ip_sock.recv_icmp() {
        if src != from {
          continue;
        }
        let mut buf = self.buf.borrow_mut();
        buf.resize(4 + icmp.payload.len(), 0);
        let pack = IcmpPacket::new(&buf).unwrap();
        let echo_reply = EchoReplyPacket::new(pack.packet()).unwrap().from_packet();
        if echo_reply.identifier == id && echo_reply.sequence_number == seq {
          return Ok(echo_reply.payload);
        }
      }
    }
    Err(std::io::ErrorKind::TimedOut.into())
  }
}

use crate::{
  packet::{compose_icmp, compose_tcp, compose_udp},
  ASockProtocol,
};
use pnet::packet::{
  icmp::*,
  ipv4::{Ipv4, MutableIpv4Packet},
  tcp::*,
  udp::*,
  Packet,
};
use socket2::{Domain, Socket, Type};
use std::{
  io::{ErrorKind, Result},
  net::{Ipv4Addr, SocketAddrV4},
};

const PACK_SIZE: usize = 4096;
const SRC_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const DEST_ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);

struct RawSock {
  send_buf: [u8; PACK_SIZE],
  sock: Socket,
}

impl RawSock {
  fn new(protocol: ASockProtocol) -> Result<Self> {
    let sock = Socket::new(Domain::IPV4, Type::RAW, Some(protocol.into()))?;
    sock.set_header_included(true)?;
    sock.bind(&SocketAddrV4::new(SRC_ADDR, 0).into())?;
    Ok(Self {
      send_buf: [0; PACK_SIZE],
      sock,
    })
  }
  fn send(&mut self, ipv4: Ipv4) -> Result<()> {
    let len = ipv4.total_length as usize;
    let dest = SocketAddrV4::new(ipv4.destination, 0);
    let mut pack = MutableIpv4Packet::new(&mut self.send_buf[..len]).ok_or(ErrorKind::InvalidData)?;
    pack.populate(&ipv4);
    self.sock.send_to(pack.packet(), &dest.into())?;

    Ok(())
  }
}

#[test]
fn icmp() -> Result<()> {
  let pack = Icmp {
    icmp_type: IcmpTypes::EchoReply,
    icmp_code: IcmpCode::new(0),
    checksum: 0,
    payload: vec![1, 2, 3, 4, 5, 6],
  };
  let mut sock = RawSock::new(ASockProtocol::ICMP)?;
  sock.send(compose_icmp(&pack, SRC_ADDR, DEST_ADDR))?;

  Ok(())
}

#[test]
fn udp() -> Result<()> {
  let pack = Udp {
    source: 13,
    destination: 14,
    length: 8 + 4,
    checksum: 0,
    payload: vec![0, 1, 2, 3],
  };
  let mut sock = RawSock::new(ASockProtocol::ICMP)?;
  sock.send(compose_udp(&pack, SRC_ADDR, DEST_ADDR))?;

  Ok(())
}

#[test]
fn tcp() -> Result<()> {
  let pack = Tcp {
    source: 41,
    destination: 23,
    sequence: 0,
    acknowledgement: 1,
    data_offset: 5,
    reserved: 0,
    flags: 0,
    window: 13,
    checksum: 0,
    urgent_ptr: 0,
    options: vec![],
    payload: vec![1, 2, 3, 3, 3, 3, 4, 5, 5, 5, 6, 6, 6, 7],
  };
  let mut sock = RawSock::new(ASockProtocol::ICMP)?;
  sock.send(compose_tcp(&pack, SRC_ADDR, DEST_ADDR))?;

  Ok(())
}

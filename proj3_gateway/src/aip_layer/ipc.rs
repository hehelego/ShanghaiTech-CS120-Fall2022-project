use crate::ASockProtocol;
use pnet::packet::{
  ipv4::{Ipv4, Ipv4Packet, MutableIpv4Packet},
  FromPacket,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use socket2::{SockAddr, Socket};
use std::{
  io::{ErrorKind, Result},
  mem,
  net::SocketAddrV4,
  time::Duration,
};

/// IPC unix domain socket send/recv timeout
pub const IPC_TIMEOUT: Duration = Duration::from_secs(1);
/// Maximum packet size for IPC packet send over unix domain socket
pub const IPC_PACK_SIZE: usize = 2048;

pub(crate) fn send_packet<T: Serialize>(socket: &Socket, addr: &SockAddr, packet: &T) -> std::io::Result<()> {
  let packet = serde_json::to_vec(packet)?;
  socket.send_to(&packet, addr)?;
  Ok(())
}
pub(crate) fn recv_packet<T: DeserializeOwned>(socket: &Socket, addr: &SockAddr) -> std::io::Result<T> {
  let mut recv_buf = vec![mem::MaybeUninit::zeroed(); IPC_PACK_SIZE];
  let (n, from_addr) = socket.recv_from(&mut recv_buf)?;
  unsafe {
    if *addr.as_ptr() != *from_addr.as_ptr() {
      return Err(ErrorKind::InvalidData.into());
    }
  }
  let buf = recv_buf[..n]
    .iter()
    .map(|x| unsafe { mem::transmute(*x) })
    .collect::<Vec<u8>>();
  serde_json::from_slice(&buf).map_err(|_| ErrorKind::InvalidData.into())
}

pub(crate) fn extract_ip_pack(response: Response) -> Result<Ipv4> {
  if let Response::ReceivedPacket(packet) = response {
    Ok(packet.into())
  } else {
    Err(ErrorKind::InvalidData.into())
  }
}

/// Wrapper of [`pnet::packet::ipv4::Ipv4`]
/// with [`serde::Serialize`] and [`serde::Deserialize`]
#[derive(Serialize, Deserialize, Debug)]
pub struct WrapIpv4(Vec<u8>);

impl From<WrapIpv4> for Ipv4 {
  fn from(bytes: WrapIpv4) -> Self {
    let packet = Ipv4Packet::new(&bytes.0).unwrap();
    packet.from_packet()
  }
}
impl From<Ipv4> for WrapIpv4 {
  fn from(ipv4: Ipv4) -> Self {
    let mut buf = vec![0; ipv4.total_length as usize];
    let mut packet = MutableIpv4Packet::new(&mut buf).unwrap();
    packet.populate(&ipv4);
    Self(buf)
  }
}

/// Accessor -> Provider IPC packet protocol.
/// Request to perform packet send or bind a socket
#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
  BindSocket(ASockProtocol, SocketAddrV4),
  UnbindSocket,
  SendPacket(WrapIpv4),
}

/// Provider -> Accessor IPC packet protocol.
/// Response to perform packet send or bind a socket
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
  BindResult(bool),
  ReceivedPacket(WrapIpv4),
}

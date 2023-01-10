use crate::{
  common::{IPC_PACK_SIZE, IPC_RETRY_WAIT},
  ASockProtocol,
};
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
  thread::sleep,
};

pub(crate) fn send_packet<T: Serialize>(socket: &Socket, addr: &SockAddr, packet: &T) {
  let mut send_buf = [0; IPC_PACK_SIZE];
  let packet = postcard::to_slice(packet, &mut send_buf).unwrap();
  while socket.send_to(packet, addr).is_err() {
    sleep(IPC_RETRY_WAIT);
    log::trace!("IPC packet send retry");
  }
}

pub(crate) fn recv_packet<T: DeserializeOwned>(socket: &Socket) -> Result<T> {
  log::trace!("IPC socket try to receive a packet",);
  let mut recv_buf = [mem::MaybeUninit::zeroed(); IPC_PACK_SIZE];
  let (n, _) = socket.recv_from(&mut recv_buf)?;
  log::debug!("IPC socket received a packet",);
  let buf = recv_buf[..n]
    .iter()
    .map(|x| unsafe { mem::transmute(*x) })
    .collect::<Vec<u8>>();
  postcard::from_bytes(&buf).map_err(|_| ErrorKind::InvalidData.into())
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct IpcPath(String);

impl IpcPath {
  pub fn new(path: &str) -> Self {
    Self(path.to_owned())
  }
  pub fn as_sockaddr(&self) -> SockAddr {
    SockAddr::unix(&self.0).unwrap()
  }
}

/// Accessor -> Provider IPC packet protocol.
/// Request to perform packet send or bind a socket
#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
  BindSocket(IpcPath, ASockProtocol, SocketAddrV4),
  UnbindSocket(IpcPath),
  SendPacket(WrapIpv4),
}

/// Provider -> Accessor IPC packet protocol.
/// Response to perform packet send or bind a socket
#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
  BindResult(bool),
  ReceivedPacket(WrapIpv4),
}

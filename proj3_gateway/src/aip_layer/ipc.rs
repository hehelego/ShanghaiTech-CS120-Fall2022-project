use crate::ASockProtocol;
use std::{net::SocketAddrV4, time::Duration};

use pnet::packet::ipv4::{Ipv4, Ipv4Packet};
use pnet::packet::FromPacket;
use serde::{Deserialize, Serialize};

/// IPC unix domain socket send/recv timeout
pub const IPC_TIMEOUT: Duration = Duration::from_secs(1);

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

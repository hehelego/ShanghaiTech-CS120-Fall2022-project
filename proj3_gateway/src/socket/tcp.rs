use crossbeam_channel::{unbounded, Receiver, Sender};
use pnet::packet::tcp::Tcp;
use std::{net::SocketAddrV4, time::Duration};

use super::ASockProtocol;

/// A socket for sending/receiving TCP packets.
/// Provides transport layer APIs.

enum TcpState {
  SynSent,
  SynReceived,
  Established,
  FinWait1,
  FinWait2,
  Closing,
  CloseWait,
  LastAck,
  Closed,
}

struct TcpStateMachine {
  state: TcpState,
  packet_assembled: Sender<u8>,
  packet_received: Receiver<Tcp>,
  bytes_to_send: Receiver<u8>,
}

pub struct TcpStream;

impl TcpStream {
  pub const PROTOCOL: ASockProtocol = ASockProtocol::TCP;

  pub fn bind(addr: SocketAddrV4) -> Result<Self, ()> {
    todo!()
  }

  pub(crate) fn transfer() -> Self {
    todo!()
  }

  pub fn connect(&self, dest: SocketAddrV4) -> Result<(), ()> {
    todo!()
  }

  pub fn shutdown(&self) -> Result<(), ()> {
    todo!()
  }
  pub fn read_timeout(&self, buf: &mut [u8], timeout: Option<Duration>) -> Result<usize, ()> {
    todo!()
  }
  pub fn write_timeout(&self, buf: &[u8], timeout: Option<Duration>) -> Result<usize, ()> {
    todo!()
  }
}

pub struct TcpListener;

impl TcpListener {
  pub const PROTOCOL: ASockProtocol = ASockProtocol::TCP;
  pub fn bind(addr: SocketAddrV4) -> Result<Self, ()> {
    todo!()
  }
  pub fn accept(&self) -> Result<(TcpStream, SocketAddrV4), ()> {
    todo!()
  }
}

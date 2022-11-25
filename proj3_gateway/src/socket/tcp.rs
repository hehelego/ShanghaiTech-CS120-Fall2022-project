use crossbeam_channel::{unbounded, Receiver, Sender};
use pnet::packet::tcp::Tcp;
use std::{
  net::SocketAddrV4,
  time::{Duration, Instant},
};

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

pub(crate) struct TcpStateMachine {
  state: TcpState,
  bytes_assembled: Sender<u8>,
  packet_received: Receiver<Tcp>,
  bytes_to_send: Receiver<u8>,
}

impl TcpStateMachine {
  pub fn new(bytes_assembled: Sender<u8>, bytes_to_send: Receiver<u8>, addr: SocketAddrV4) -> Self {
    todo!()
  }
  pub fn connect(&self) {
    todo!()
  }
  pub fn sync(&self, dest: SocketAddrV4) -> Result<(), ()> {
    todo!()
  }
  pub fn shutdown(&self) -> Result<(), ()> {
    todo!()
  }
}

pub struct TcpStream {
  bytes_to_send: Sender<u8>,
  bytes_assembled: Receiver<u8>,
  state_machine: TcpStateMachine,
}

impl TcpStream {
  pub const PROTOCOL: ASockProtocol = ASockProtocol::TCP;

  pub fn bind(addr: SocketAddrV4) -> Result<Self, ()> {
    let (bytes_assembled_tx, bytes_assmebled_rx) = crossbeam_channel::unbounded();
    let (bytes_to_send_tx, bytes_to_send_rx) = crossbeam_channel::unbounded();
    // Create the Tcp State Machine
    let state_machine = TcpStateMachine::new(bytes_assembled_tx, bytes_to_send_rx, addr);
    // Tcp StateMachine
    Ok(Self {
      bytes_to_send: bytes_to_send_tx,
      bytes_assembled: bytes_assmebled_rx,
      state_machine,
    })
  }

  pub(crate) fn transfer(
    bytes_to_send: Sender<u8>,
    bytes_assembled: Receiver<u8>,
    state_machine: TcpStateMachine,
  ) -> Self {
    Self {
      bytes_to_send,
      bytes_assembled,
      state_machine,
    }
  }

  pub fn connect(&mut self, dest: SocketAddrV4) -> Result<(), ()> {
    self.state_machine.sync(dest)
  }

  pub fn shutdown(&self) -> Result<(), ()> {
    self.state_machine.shutdown()
  }
  pub fn read_timeout(&self, buf: &mut [u8], timeout: Option<Duration>) -> Result<usize, ()> {
    let mut bytes_read = 0;
    if let Some(timeout) = timeout {
      let deadline = Instant::now() + timeout;
      for x in buf.iter_mut() {
        match self.bytes_assembled.recv_deadline(deadline) {
          Ok(byte) => {
            bytes_read += 1;
            *x = byte
          }
          Err(_) => return Ok(bytes_read),
        }
      }
    } else {
      for x in buf.iter_mut() {
        match self.bytes_assembled.recv() {
          Ok(byte) => {
            *x = byte;
            bytes_read += 1;
          }
          Err(_) => return Ok(bytes_read),
        }
      }
    }
    Ok(bytes_read)
  }
  pub fn write_timeout(&self, buf: &[u8], timeout: Option<Duration>) -> Result<usize, ()> {
    let mut byte_writes = 0;
    if let Some(timeout) = timeout {
      let deadline = Instant::now() + timeout;
      for x in buf.iter() {
        match self.bytes_to_send.send_deadline(*x, deadline) {
          Ok(_) => byte_writes += 1,
          Err(_) => return Ok(byte_writes),
        }
      }
    } else {
      for x in buf.iter() {
        match self.bytes_to_send.send(*x) {
          Ok(_) => byte_writes += 1,
          Err(_) => return Ok(byte_writes),
        }
      }
    }
    Ok(byte_writes)
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

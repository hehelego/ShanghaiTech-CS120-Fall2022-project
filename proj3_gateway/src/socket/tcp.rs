//! The tcp API. This module provide two types of the tcp socket: TcpStream and TcpListner.
//! TcpStream is used for client and TcpListner is for the server.

use crossbeam_channel::{Receiver, Sender};
use std::{
  net::SocketAddrV4,
  time::{Duration, Instant},
};

use super::ASockProtocol;
use tcp_state_machine::TcpStateMachine;

/// A socket for sending/receiving TCP packets.
/// Provides transport layer APIs.
pub struct TcpStream {
  bytes_to_send: Sender<u8>,
  bytes_assembled: Receiver<u8>,
  state_machine: TcpStateMachine,
}

impl TcpStream {
  pub const PROTOCOL: ASockProtocol = ASockProtocol::TCP;
  /// Bind the TcpStream to a local addresss.
  /// Returns a TcpStream if success.
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

  // This function is for TcpListner.
  pub(self) fn transfer(
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
  /// Connect the TcpStream to a remote address.
  /// Return Error if connections timeout.
  pub fn connect(&mut self, dest: SocketAddrV4) -> Result<(), ()> {
    const HAND_SHAKE_MAX_TIME: Duration = Duration::from_secs(6);
    self.state_machine.connect(dest)?;
    self
      .bytes_assembled
      .recv_deadline(Instant::now() + HAND_SHAKE_MAX_TIME)
      .map_err(|_| ())?;
    Ok(())
  }

  /// Shutdown the TcpStream
  pub fn shutdown(&self) -> Result<(), ()> {
    self.state_machine.shutdown()
  }

  /// Try to read data to buf. Wait for at most `timeout` time. If `timeout` is None, the function is blocking.
  /// Returns the length of data read on success.
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
      // blocking
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

  /// Try to write data from buf. Wait for at most `timeout` time. If `timeout` is None, the function is blocking.
  /// Returns the length of data written on success.
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
      // blocking
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

pub mod tcp_state_machine;
pub mod wrapping_integers;

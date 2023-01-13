//! The tcp API. This module provide two types of the tcp socket: TcpStream and TcpListener.
//! TcpStream is used for client and TcpListener is for the server.

use crossbeam_channel::{Receiver, Sender};
use pnet::packet::tcp::{Tcp, TcpFlags};
use std::{
  collections::HashMap,
  net::SocketAddrV4,
  thread::{self, JoinHandle},
  time::{Duration, Instant},
};

use crate::IpAccessor;

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
    log::debug!("[Tcp Stream] bind to {}", addr);
    Ok(Self {
      bytes_to_send: bytes_to_send_tx,
      bytes_assembled: bytes_assmebled_rx,
      state_machine,
    })
  }

  /// TCP TcpListener control transfer for accepting connections
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
    const HAND_SHAKE_MAX_TIME: Duration = Duration::from_secs(30);
    self.state_machine.connect(dest)?;
    self
      .bytes_assembled
      .recv_deadline(Instant::now() + HAND_SHAKE_MAX_TIME)
      .map_err(|_| ())?;
    Ok(())
  }

  /// Shutdown the TcpStream
  pub fn shutdown_both(&self) -> Result<(), ()> {
    self.state_machine.shutdown_read()?;
    self.state_machine.shutdown_write()
  }

  pub fn shutdown_read(&self) -> Result<(), ()> {
    self.state_machine.shutdown_read()
  }

  pub fn shutdown_write(&self) -> Result<(), ()> {
    self.state_machine.shutdown_write()
  }

  /// Try to read data to buf. Wait for at most `timeout` time. If `timeout` is None, the function is blocking.
  /// Returns the length of data read on success.
  pub fn read_timeout(&self, buf: &mut [u8], timeout: Option<Duration>) -> (usize, bool) {
    let mut bytes_read = 0;
    if let Some(timeout) = timeout {
      let deadline = Instant::now() + timeout;
      for x in buf.iter_mut() {
        match self.bytes_assembled.recv_deadline(deadline) {
          Ok(byte) => {
            bytes_read += 1;
            *x = byte
          }
          Err(e) => match e {
            crossbeam_channel::RecvTimeoutError::Timeout => return (bytes_read, false),
            crossbeam_channel::RecvTimeoutError::Disconnected => return (bytes_read, true),
          },
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
          Err(_) => return (bytes_read, true),
        }
      }
    }
    (bytes_read, false)
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

pub struct TcpListener {
  dispatcher_handle: Option<JoinHandle<()>>,
  terminate_signal: Sender<()>,
  pending_connection: Receiver<(
    Tcp,
    SocketAddrV4,
    Sender<(Tcp, SocketAddrV4)>,
    Receiver<(Tcp, SocketAddrV4)>,
    Sender<()>,
  )>,
  src_addr: SocketAddrV4,
}

impl TcpListener {
  pub const PROTOCOL: ASockProtocol = ASockProtocol::TCP;
  pub fn bind(addr: SocketAddrV4) -> Result<Self, ()> {
    let (terminate_signal_tx, terminate_signal_rx) = crossbeam_channel::unbounded();
    let (pending_connection_tx, pending_connection_rx) = crossbeam_channel::unbounded();
    let dispatcher = thread::spawn(move || {
      let mut worker = TcpListenerWorker::new(addr, terminate_signal_rx, pending_connection_tx);
      worker.run()
    });
    Ok(Self {
      dispatcher_handle: Some(dispatcher),
      terminate_signal: terminate_signal_tx,
      pending_connection: pending_connection_rx,
      src_addr: addr,
    })
  }
  pub fn accept(&self) -> Result<(TcpStream, SocketAddrV4), ()> {
    let (bytes_to_send_tx, bytes_to_send_rx) = crossbeam_channel::unbounded();
    let (bytes_assembled_tx, bytes_assembled_rx) = crossbeam_channel::unbounded();
    match self.pending_connection.recv() {
      Ok((packet, addr, packet_to_send, packet_received, access_termination_signal)) => {
        let state_machine = TcpStateMachine::syn_received(
          self.src_addr,
          addr,
          packet,
          bytes_assembled_tx,
          packet_to_send,  // receive from the channel
          packet_received, // receive from the channel
          bytes_to_send_rx,
          access_termination_signal, // receive from the channel
        );
        let tcp_stream = TcpStream::transfer(bytes_to_send_tx, bytes_assembled_rx, state_machine);
        Ok((tcp_stream, addr))
      }
      Err(e) => {
        log::debug!("[Listener] Accept error: {}", e);
        Err(())
      }
    }
  }
}

impl Drop for TcpListener {
  fn drop(&mut self) {
    self.terminate_signal.send(()).unwrap();
    if let Some(handle) = self.dispatcher_handle.take() {
      handle.join().unwrap();
    }
  }
}

pub struct TcpListenerWorker {
  terminate_signal_rx: Receiver<()>,
  packet_channel: (Sender<(Tcp, SocketAddrV4)>, Receiver<(Tcp, SocketAddrV4)>),
  accessor: IpAccessor,
  hash_map: HashMap<SocketAddrV4, (Sender<(Tcp, SocketAddrV4)>, Receiver<()>)>,
  pending_connection: Sender<(
    Tcp,
    SocketAddrV4,
    Sender<(Tcp, SocketAddrV4)>,
    Receiver<(Tcp, SocketAddrV4)>,
    Sender<()>,
  )>,
}
impl TcpListenerWorker {
  fn new(
    addr: SocketAddrV4,
    terminate_signal_rx: Receiver<()>,
    connection_sender: Sender<(
      Tcp,
      SocketAddrV4,
      Sender<(Tcp, SocketAddrV4)>,
      Receiver<(Tcp, SocketAddrV4)>,
      Sender<()>,
    )>,
  ) -> Self {
    let path = format!("/tmp/tcp_listener_{}", addr);
    let accessor = IpAccessor::new(&path).unwrap();
    accessor.bind(ASockProtocol::TCP, addr).unwrap();
    let hash_map = HashMap::new();
    log::debug!("[Listener Worker] start");
    Self {
      terminate_signal_rx,
      packet_channel: crossbeam_channel::unbounded(),
      accessor,
      hash_map,
      pending_connection: connection_sender,
    }
  }
  fn run(&mut self) {
    while self.terminate_signal_rx.try_recv().is_err() {
      if let Ok((packet, addr)) = self.accessor.recv_tcp() {
        // Check the addr and dispatch
        self.dispatch(packet, addr)
      }
      if let Ok((packet, addr)) = self.packet_channel.1.try_recv() {
        self.accessor.send_tcp(packet, addr).unwrap()
      }
      // Clear up
      self.hash_map.retain(|_, v| v.1.try_recv().is_err())
    }
    println!("[Listener Worker] end");
  }
  fn dispatch(&mut self, packet: Tcp, addr: SocketAddrV4) {
    if let Some((sender, signal)) = self.hash_map.get(&addr) {
      // dispatch
      if signal.try_recv().is_err() {
        let _ = sender.send((packet, addr));
      }
    } else if packet.flags & TcpFlags::SYN != 0 {
      // Create an entry
      let (packet_tx, packet_rx) = crossbeam_channel::unbounded();
      let (termination_tx, termination_rx) = crossbeam_channel::unbounded();
      // Insert the key into the hash map
      self.hash_map.insert(addr, (packet_tx, termination_rx));
      // Send the connection information
      self
        .pending_connection
        .send((packet, addr, self.packet_channel.0.clone(), packet_rx, termination_tx))
        .unwrap()
    }
  }
}

pub mod tcp_state_machine;

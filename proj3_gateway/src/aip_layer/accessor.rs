use crate::ASockProtocol;
use crossbeam_channel::{unbounded, Receiver, Sender};
use pnet::packet::{icmp::Icmp, ipv4::Ipv4, tcp::Tcp, udp::Udp};
use socket2::{Domain, SockAddr, Socket, Type};
use std::cell::RefCell;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use std::thread::{spawn, JoinHandle};


struct Worker {
  handler: Option<JoinHandle<()>>,
  pack_rx: Receiver<Ipv4>,
  exit_tx: Sender<()>,
}

impl Worker {
  fn work(sock: Arc<Socket>, pack_tx: Sender<Ipv4>, exit_rx: Receiver<()>) {
    // TODO: repeatedly receive packets from IP provider server
    todo!()
  }
  fn start(sock: Arc<Socket>) -> Self {
    let (exit_tx, exit_rx) = unbounded();
    let (pack_tx, pack_rx) = unbounded();
    let handler = Some(spawn(move || Self::work(sock, pack_tx, exit_rx)));
    Self {
      handler,
      pack_rx,
      exit_tx,
    }
  }
}
impl Drop for Worker {
  fn drop(&mut self) {
    if let Some(handler) = self.handler.take() {
      self.exit_tx.send(()).unwrap();
      handler.join().unwrap();
    }
  }
}

/// Athernet IP service accessor, used to communicate with the provider process.
/// An [`IpAccessor`] should be associated with a unique Athernet socket object.
pub struct IpAccessor {
  ipc_path: String,
  sock: Arc<Socket>,
  worker: RefCell<Option<Worker>>,
}

impl IpAccessor {
  /// create an [`IpAccessor`] object.
  /// The accessor object will create a unix domain socket on `sock_path`
  /// and communicate via [`IpProvider`] which is listening on [`IpProvider::SOCK_PATH`].
  pub fn new(sock_path: &str) -> std::io::Result<Self> {
    // unix domain socket creation
    let _ = std::fs::remove_file(sock_path);
    let sock = Socket::new(Domain::UNIX, Type::DGRAM, None)?;
    sock.bind(&SockAddr::unix(sock_path)?)?;
    Ok(Self {
      ipc_path: sock_path.to_owned(),
      sock: Arc::new(sock),
      worker: RefCell::new(None),
    })
  }

  /// Try to bind a socket to a specific address.
  /// - Return `true` on success and `false` on error.
  /// - This function will negotiate with [`IpProvider`] for resource allocation and mapping construction.
  ///   The socket is unbinded when this accessor object is dropped.
  /// - This function also create a separated thread running in the background to repeatedly receive
  ///   packets comming from the [`IpProvider`].
  ///   The worker is terminated when the accessor object is dropped.
  pub fn bind(&self, sock_type: ASockProtocol, sock_addr: SocketAddrV4) -> bool {
    if self.worker.borrow().is_some() {
      return false;
    }
    // TODO: communicate with provider
    todo!();
    // create worker
    *self.worker.borrow_mut() = Some(Worker::start(self.sock.clone()));
    true
  }
  fn unbind(&self) {
    if let Some(worker) = self.worker.borrow_mut().take() {
      drop(worker);
    }
  }
}

impl Drop for IpAccessor {
  fn drop(&mut self) {
    self.unbind();
  }
}

/// ICMP
impl IpAccessor {
  /// Send an ICMP packet via the network layer.
  pub fn send_icmp(&self, packet: Icmp, dest: Ipv4Addr) {
    todo!()
  }
  /// Receive an ICMP packet from the network layer.
  /// The ICMP representation packet and the source address are returned.
  pub fn recv_icmp(&self) -> (Icmp, Ipv4Addr) {
    todo!()
  }
}

/// TCP
impl IpAccessor {
  /// Send a TCP packet via the network layer.
  pub fn send_tcp(&self, packet: Tcp, dest: SocketAddrV4) {
    todo!()
  }
  /// Receive a TCP packet from the network layer.
  /// The TCP representation packet and the source address are returned.
  pub fn recv_tcp(&self) -> (Tcp, SocketAddrV4) {
    todo!()
  }
}

/// UDP
impl IpAccessor {
  /// Send a UDP packet via the network layer.
  pub fn send_udp(&self, packet: Udp, dest: SocketAddrV4) {
    todo!()
  }
  /// Receive a UDP packet from the network layer.
  /// The TCP representation packet and the source address are returned.
  pub fn recv_udp(&self) -> (Udp, SocketAddrV4) {
    todo!()
  }
}


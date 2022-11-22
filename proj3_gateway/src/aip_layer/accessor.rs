use crate::{
  aip_layer::ipc::{extract_ip_pack, recv_packet, send_packet, Request, Response},
  common::IPC_TIMEOUT,
  packet::{compose_icmp, compose_tcp, compose_udp, parse_icmp, parse_tcp, parse_udp},
  ASockProtocol, IpProvider,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use pnet::packet::{icmp::Icmp, ipv4::Ipv4, tcp::Tcp, udp::Udp};
use socket2::{Domain, SockAddr, Socket, Type};
use std::{
  cell::RefCell,
  io::{ErrorKind, Result},
  net::{Ipv4Addr, SocketAddrV4},
  sync::Arc,
  thread::{spawn, JoinHandle},
};

struct Worker {
  handler: Option<JoinHandle<()>>,
  pack_rx: Receiver<Ipv4>,
  exit_tx: Sender<()>,
}

impl Worker {
  /// IP accessor worker thread main function:
  /// repeatedly received [`ipc::Response`] packets from the provider.
  fn work(sock: Arc<Socket>, pack_tx: Sender<Ipv4>, exit_rx: Receiver<()>) {
    while exit_rx.try_recv().is_err() {
      match recv_packet::<Response>(&sock, &IpProvider::sock_path()) {
        // probability received a response packet, try to extract the append result
        Ok(resp) => {
          if let Ok(packet) = extract_ip_pack(resp) {
            pack_tx.send(packet).unwrap();
          }
        }
        // timeout, no packet is received, wait for future packets
        Err(e) if e.kind() == ErrorKind::TimedOut => continue,
        // other error occurred, exit
        _ => break,
      }
    }
  }
  /// prepare and start a IP accessor worker thread
  /// the worker thread stop when the worker object is dropped.
  ///
  /// This function returns immediately with an handler of the worker thread.
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
  fn recv_channel(&self) -> &Receiver<Ipv4> {
    &self.pack_rx
  }
}
impl Drop for Worker {
  /// notify and stop the worker thread
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
  sock: Arc<Socket>,
  worker: RefCell<Option<Worker>>,
  bind_addr: RefCell<Option<Ipv4Addr>>,
}

impl IpAccessor {
  /// create an [`IpAccessor`] object.
  /// The accessor object will create a unix domain socket on `sock_path`
  /// and communicate via [`IpProvider`] which is listening on [`IpProvider::SOCK_PATH`].
  pub fn new(sock_path: &str) -> Result<Self> {
    // unix domain socket creation
    let _ = std::fs::remove_file(sock_path);
    let sock = Socket::new(Domain::UNIX, Type::DGRAM, None)?;
    sock.set_read_timeout(Some(IPC_TIMEOUT))?;
    sock.set_write_timeout(Some(IPC_TIMEOUT))?;
    sock.bind(&SockAddr::unix(sock_path)?)?;
    Ok(Self {
      sock: Arc::new(sock),
      worker: RefCell::new(None),
      bind_addr: RefCell::new(None),
    })
  }

  /// Try to bind a socket to a specific address.
  /// - Return `true` on success and `false` on error.
  /// - This function will negotiate with [`IpProvider`] for resource allocation and mapping construction.
  ///   The socket is unbinded when this accessor object is dropped.
  /// - This function also create a separated thread running in the background to repeatedly receive
  ///   packets comming from the [`IpProvider`].
  ///   The worker is terminated when the accessor object is dropped.
  pub fn bind(&self, sock_type: ASockProtocol, sock_addr: SocketAddrV4) -> Result<()> {
    // already bind
    if self.bind_addr.borrow().is_some() {
      return Err(ErrorKind::AddrInUse.into());
    }
    // communicate with provider to bind
    let bind_request = Request::BindSocket(sock_type, sock_addr);
    send_packet(&self.sock, &IpProvider::sock_path(), &bind_request)?;
    let bind_response = recv_packet(&self.sock, &IpProvider::sock_path())?;
    // expecting a correct response
    match bind_response {
      Response::BindResult(true) => {
        // create worker
        *self.worker.borrow_mut() = Some(Worker::start(self.sock.clone()));
        *self.bind_addr.borrow_mut() = Some(*sock_addr.ip());
        Ok(())
      }
      _ => Err(ErrorKind::AddrInUse.into()),
    }
  }
  /// Send an unbind packet to the IP provider to unbind this socket,
  /// also stop the worker thread
  fn unbind(&self) -> Result<()> {
    if let Some(worker) = self.worker.borrow_mut().take() {
      let unbind_request = Request::UnbindSocket;
      send_packet(&self.sock, &IpProvider::sock_path(), &unbind_request)?;
      drop(worker);
    }
    *self.bind_addr.borrow_mut() = None;
    Ok(())
  }
}

impl Drop for IpAccessor {
  fn drop(&mut self) {
    let _ = self.unbind();
  }
}

/// ICMP
impl IpAccessor {
  /// Send an ICMP packet via the network layer.
  pub fn send_icmp(&self, packet: Icmp, dest: Ipv4Addr) -> Result<()> {
    let ipv4 = compose_icmp(&packet, self.bind_addr.borrow().unwrap(), dest);
    let send_req = Request::SendPacket(ipv4.into());
    send_packet(&self.sock, &IpProvider::sock_path(), &send_req)?;
    Ok(())
  }
  /// Receive an ICMP packet from the network layer.
  /// The ICMP representation packet and the source address are returned.
  pub fn recv_icmp(&self) -> Result<(Icmp, Ipv4Addr)> {
    let resp: Response = recv_packet(&self.sock, &IpProvider::sock_path())?;
    let ipv4 = extract_ip_pack(resp)?;
    let icmp = parse_icmp(&ipv4).ok_or(ErrorKind::InvalidData)?;
    Ok((icmp, ipv4.source))
  }
}

/// TCP
impl IpAccessor {
  /// Send a TCP packet via the network layer.
  pub fn send_tcp(&self, packet: Tcp, dest: SocketAddrV4) -> Result<()> {
    let ipv4 = compose_tcp(&packet, self.bind_addr.borrow().unwrap(), *dest.ip());
    let send_req = Request::SendPacket(ipv4.into());
    send_packet(&self.sock, &IpProvider::sock_path(), &send_req)?;
    Ok(())
  }
  /// Receive a TCP packet from the network layer.
  /// The TCP representation packet and the source address are returned.
  pub fn recv_tcp(&self) -> Result<(Tcp, SocketAddrV4)> {
    let resp: Response = recv_packet(&self.sock, &IpProvider::sock_path())?;
    let ipv4 = extract_ip_pack(resp)?;
    let tcp = parse_tcp(&ipv4).ok_or(ErrorKind::InvalidData)?;
    let src_addr = SocketAddrV4::new(ipv4.source, tcp.source);
    Ok((tcp, src_addr))
  }
}

/// UDP
impl IpAccessor {
  /// Send a UDP packet via the network layer.
  pub fn send_udp(&self, packet: Udp, dest: SocketAddrV4) -> Result<()> {
    let ipv4 = compose_udp(&packet, self.bind_addr.borrow().unwrap(), *dest.ip());
    let send_req = Request::SendPacket(ipv4.into());
    send_packet(&self.sock, &IpProvider::sock_path(), &send_req)?;
    Ok(())
  }
  /// Receive a UDP packet from the network layer.
  /// The TCP representation packet and the source address are returned.
  pub fn recv_udp(&self) -> Result<(Udp, SocketAddrV4)> {
    let resp: Response = recv_packet(&self.sock, &IpProvider::sock_path())?;
    let ipv4 = extract_ip_pack(resp)?;
    let udp = parse_udp(&ipv4).ok_or(ErrorKind::InvalidData)?;
    let src_addr = SocketAddrV4::new(ipv4.source, udp.source);
    Ok((udp, src_addr))
  }
}

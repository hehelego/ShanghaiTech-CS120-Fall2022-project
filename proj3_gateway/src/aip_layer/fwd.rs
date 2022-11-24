use crate::{
  aip_layer::ipc::{recv_packet, send_packet, IpcPath, Request, Response},
  common::{aip_ipc_sockaddr, AIP_SOCK, IPC_TIMEOUT},
  packet::{parse_icmp, parse_tcp, parse_udp, IpOverMac},
  ASockProtocol,
};
use pnet::packet::ipv4::Ipv4;
use proj2_multiple_access::MacAddr;
use socket2::{Domain, Socket, Type};
use std::{
  collections::HashMap,
  io::Result,
  net::{Ipv4Addr, SocketAddrV4},
};

/// A unix-domain socket server which provides IP layer service:
/// send IP packets to peer or receive IP packets from peer
///
/// This is the IP layer for Athernet internal node
///
/// **NOTE** On every Athernet node, only one Provider instance should exists.
///
/// - hold an unique mac layer object
/// - communicate with other processes to perform
///   - IP packet send/receive
///   - socket bind/unbind
///   via unix domain socket IPC
pub struct IpLayerInternal {
  // L2/L3 address
  self_ip: Ipv4Addr,
  _peer_ip: Ipv4Addr,
  // send/recv IPv4 packets via MAC
  ip_txrx: IpOverMac,
  // IPC
  ipc: Socket,
  // socket in use: sock-addr <-> IPC socket
  socks_in_use: HashMap<IpcPath, (ASockProtocol, SocketAddrV4)>,
  icmp_binds: HashMap<Ipv4Addr, IpcPath>,
  tcp_binds: HashMap<SocketAddrV4, IpcPath>,
  udp_binds: HashMap<SocketAddrV4, IpcPath>,
}

impl IpLayerInternal {
  /// called on receiving and IPv4 packet from peer IP layer:
  /// forward the packet to a process for further handling
  fn on_recv_ipv4(&mut self, ipv4: Ipv4) {
    // some thing wrong, receiving a packet not for us
    if ipv4.destination != self.self_ip {
      return;
    }

    // determine protocol, find the destination process
    match ipv4.next_level_protocol.try_into() {
      Ok(ASockProtocol::UDP) => parse_udp(&ipv4).and_then(|udp| {
        let addr = SocketAddrV4::new(ipv4.destination, udp.destination);
        self.udp_binds.get(&addr)
      }),
      Ok(ASockProtocol::ICMP) => parse_icmp(&ipv4).and_then(|_| self.icmp_binds.get(&ipv4.destination)),
      Ok(ASockProtocol::TCP) => parse_tcp(&ipv4).and_then(|tcp| {
        let addr = SocketAddrV4::new(ipv4.destination, tcp.destination);
        self.tcp_binds.get(&addr)
      }),
      _ => None,
    }
    // forward the IPv4 packet to that process for further handling
    .and_then(|ipc_path| {
      let pack = Response::ReceivedPacket(ipv4.into());
      let _ = send_packet(&self.ipc, &ipc_path.as_sockaddr(), &pack);
      Some(())
    });
  }
  /// called on bind socket failed: response with error message
  fn on_bind_failed(&self, ipc_path: IpcPath) {
    // TODO: error message
    let pack = Response::BindResult(false);
    let _ = send_packet(&self.ipc, &ipc_path.as_sockaddr(), &pack);
  }
  /// called on a process request to bind a socket:
  /// try to add the (socket address <-> IPC socket) mapping.
  /// send back response.
  fn handle_bind(&mut self, protocol: ASockProtocol, addr: SocketAddrV4, ipc_path: IpcPath) {
    log::debug!("handle bind request {:?}@{:?} from {:?}", protocol, addr, ipc_path);
    if *addr.ip() != self.self_ip || self.socks_in_use.contains_key(&ipc_path) {
      log::debug!("bind failed: not local address");
      self.on_bind_failed(ipc_path);
      return;
    }
    let pack = Response::BindResult(true);
    self.socks_in_use.insert(ipc_path.clone(), (protocol, addr));
    let _ = send_packet(&self.ipc, &ipc_path.as_sockaddr(), &pack);
    log::debug!("bind success");

    let _ = match protocol {
      ASockProtocol::UDP => self.udp_binds.insert(addr, ipc_path),
      ASockProtocol::ICMP => self.icmp_binds.insert(*addr.ip(), ipc_path),
      ASockProtocol::TCP => self.tcp_binds.insert(addr, ipc_path),
    };
  }
  /// called on a process request to bind a socket:
  /// try to remove the (socket address <-> IPC socket) mapping.
  fn handle_unbind(&mut self, ipc_path: IpcPath) {
    log::debug!("handle unbind request from {:?}", ipc_path);
    if let Some((protocol, addr)) = self.socks_in_use.remove(&ipc_path) {
      log::debug!("unbind {:?}@{:?}", protocol, addr);
      let _ = match protocol {
        ASockProtocol::UDP => self.udp_binds.remove(&addr),
        ASockProtocol::ICMP => self.icmp_binds.remove(addr.ip()),
        ASockProtocol::TCP => self.tcp_binds.remove(&addr),
      };
    }
  }
  /// called on a process request to send a IPv4 packet to peer:
  /// send over mac
  fn handle_send(&mut self, ipv4: Ipv4) {
    log::debug!("handle send Ipv4 {:?} -> {:?}", ipv4.source, ipv4.destination);
    self.ip_txrx.send(&ipv4);
  }

  fn mainloop(&mut self) {
    self.ip_txrx.send_poll();
    let maybe_ipv4 = self.ip_txrx.recv_poll();
    // on receiving IPv4 packet from peer
    if let Some(ipv4) = maybe_ipv4 {
      log::debug!("recv ipv4 from {:?} -> {:?}", ipv4.source, ipv4.destination);
      self.on_recv_ipv4(ipv4);
    }

    // handling requests: bind socket & send packet
    if let Ok(request) = recv_packet::<Request>(&self.ipc) {
      match request {
        Request::BindSocket(ipc_path, protocol, addr) => self.handle_bind(protocol, addr, ipc_path),
        Request::UnbindSocket(ipc_path) => self.handle_unbind(ipc_path),
        Request::SendPacket(ipv4) => self.handle_send(ipv4.into()),
      }
    }
  }

  /// Run as a internal node: forward IP packets to gateway
  pub fn run(&mut self) {
    loop {
      log::trace!("internal forward mainloop iteration");
      self.mainloop()
    }
  }

  /// Build IP layer on top of MAC layer.
  /// The MAC address and IP address should be given.
  ///
  /// - `self_addr`: the MAC address and IP address of current node
  /// - `peer_addr`: the MAC address and IP address of peer node
  pub fn new(self_addr: (MacAddr, Ipv4Addr), peer_addr: (MacAddr, Ipv4Addr)) -> Result<Self> {
    log::debug!(
      "starting IP layer for internal@{:?}, gateway@{:?}",
      self_addr,
      peer_addr
    );

    // IPC unix domain socket
    let _ = std::fs::remove_file(AIP_SOCK);
    let ipc = Socket::new(Domain::UNIX, Type::DGRAM, None)?;
    ipc.set_read_timeout(Some(IPC_TIMEOUT))?;
    ipc.set_write_timeout(Some(IPC_TIMEOUT))?;
    ipc.bind(&aip_ipc_sockaddr())?;

    Ok(Self {
      self_ip: self_addr.1,
      _peer_ip: peer_addr.1,
      ip_txrx: IpOverMac::new(self_addr.0, peer_addr.0),
      ipc,
      socks_in_use: Default::default(),
      icmp_binds: Default::default(),
      tcp_binds: Default::default(),
      udp_binds: Default::default(),
    })
  }
}

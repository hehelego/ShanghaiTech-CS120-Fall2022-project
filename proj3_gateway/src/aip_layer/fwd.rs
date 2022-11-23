use crate::{
  common::{aip_ipc_sockaddr, AIP_SOCK, IPC_TIMEOUT},
  packet::IpOverMac,
  ASockProtocol,
};
use pnet::packet::ipv4::Ipv4;
use proj2_multiple_access::{MacAddr, MacLayer};
use socket2::{Domain, SockAddr, Socket, Type};
use std::{
  collections::HashMap,
  io::Result,
  net::{Ipv4Addr, SocketAddrV4},
  path::Path,
};

use super::ipc::{recv_pack_addr, recv_packet, Request, Response};

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
  peer_ip: Ipv4Addr,
  // send/recv IPv4 packets via MAC
  ip_txrx: IpOverMac,
  // IPC
  ipc: Socket,
  // socket in use: sock-addr <-> IPC socket
  socket_in_use: HashMap<String, (ASockProtocol, SocketAddrV4)>,
  icmp_socks: HashMap<Ipv4Addr, Socket>,
  tcp_socks: HashMap<SocketAddrV4, Socket>,
  udp_socks: HashMap<SocketAddrV4, Socket>,
}

impl IpLayerInternal {
  fn on_recv_ipv4(&mut self, ipv4: &Ipv4) {
    match ipv4.next_level_protocol.try_into() {
      Ok(ASockProtocol::UDP) => todo!(),
      Ok(ASockProtocol::ICMP) => todo!(),
      Ok(ASockProtocol::TCP) => todo!(),
      _ => todo!(),
    };
  }
  fn handle_bind(&mut self, protocol: ASockProtocol, addr: SocketAddrV4, ipc_path: &Path) {
    match protocol {
      ASockProtocol::UDP => todo!(),
      ASockProtocol::ICMP => todo!(),
      ASockProtocol::TCP => todo!(),
    }
  }
  fn handle_unbind(&mut self, ipc_path: &Path) {
    todo!()
  }
  fn handle_send(&mut self, ipv4: &Ipv4) {
    todo!()
  }

  fn mainloop(&mut self) {
    // MAC ip txrx
    self.ip_txrx.send_poll();
    let maybe_ipv4 = self.ip_txrx.recv_poll();
    // on receiving IPv4 packet from peer
    if let Some(ipv4) = maybe_ipv4 {
      self.on_recv_ipv4(&ipv4);
    }

    // handling requests: bind socket & send packet
    // recv_packet(&self.ipc, None);
    if let Ok((request, from)) = recv_pack_addr::<Request>(&self.ipc) {
      match request {
        Request::BindSocket(ipc_path, protocol, addr) => self.handle_bind(protocol, addr, &ipc_path),
        Request::UnbindSocket(ipc_path) => self.handle_unbind(&ipc_path),
        Request::SendPacket(ipv4) => self.handle_send(&ipv4.into()),
      }
    }
  }

  /// Run as a internal node: forward IP packets to gateway
  pub fn run(&mut self) {
    loop {
      self.mainloop()
    }
  }

  /// Build IP layer on top of MAC layer.
  /// The MAC address and IP address should be given.
  ///
  /// - `self_addr`: the MAC address and IP address of current node
  /// - `peer_addr`: the MAC address and IP address of peer node
  pub fn new(self_addr: (MacAddr, Ipv4Addr), peer_addr: (MacAddr, Ipv4Addr)) -> Result<Self> {
    // IPC unix domain socket
    let _ = std::fs::remove_file(AIP_SOCK);
    let ipc = Socket::new(Domain::UNIX, Type::DGRAM, None)?;
    ipc.set_read_timeout(Some(IPC_TIMEOUT))?;
    ipc.set_write_timeout(Some(IPC_TIMEOUT))?;
    ipc.bind(&aip_ipc_sockaddr())?;

    Ok(Self {
      self_ip: self_addr.1,
      peer_ip: peer_addr.1,
      ip_txrx: IpOverMac::new(self_addr.0, peer_addr.0),
      ipc,
      socket_in_use: Default::default(),
      icmp_socks: Default::default(),
      tcp_socks: Default::default(),
      udp_socks: Default::default(),
    })
  }
}

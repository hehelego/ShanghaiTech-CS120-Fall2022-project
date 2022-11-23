use crate::common::{AIP_SOCK, IPC_TIMEOUT, RAWSOCK_TIMEOUT, aip_ipc_sockaddr};
use crate::packet::IpPackFrag;
use proj2_multiple_access::{MacAddr, MacLayer};
use socket2::{Domain, SockAddr, Socket, Type};
use std::{
  collections::VecDeque,
  io::Result,
  net::{Ipv4Addr, SocketAddrV4},
};


/// A unix-domain socket server which provides IP layer service:
/// send IP packets to peer or receive IP packets from peer
///
/// This is the IP layer for Athernet gateway node
///
/// **NOTE** On every Athernet node, only one Provider instance should exists.
///
/// - hold an unique mac layer object
/// - communicate with other processes to perform
///   - IP packet send/receive
///   - socket bind/unbind
///   via unix domain socket IPC
/// - route packets into/out-of the external LAN
pub struct IpLayerGateway;

impl IpLayerGateway {
  /// Build IP layer on top of MAC layer.
  /// The MAC address and IP address should be given.
  ///
  /// Run as a gateway node: NAT
  ///
  /// - `self_addr`: the MAC address and IP address of current node
  /// - `peer_addr`: the MAC address and IP address of peer node
  pub fn run(self_addr: (MacAddr, Ipv4Addr), peer_addr: (MacAddr, Ipv4Addr)) -> Result<()> {
    let (self_mac, self_ip) = self_addr;
    let (peer_mac, peer_ip) = peer_addr;

    // IPC unix domain socket
    let _ = std::fs::remove_file(AIP_SOCK);
    let ipc = Socket::new(Domain::UNIX, Type::DGRAM, None)?;
    ipc.set_read_timeout(Some(IPC_TIMEOUT))?;
    ipc.set_write_timeout(Some(IPC_TIMEOUT))?;
    ipc.bind(&aip_ipc_sockaddr())?;

    // RAW IP packet socket creation
    let rawip = Socket::new(Domain::IPV4, Type::RAW, None)?;
    rawip.set_read_timeout(Some(RAWSOCK_TIMEOUT))?;
    rawip.set_write_timeout(Some(RAWSOCK_TIMEOUT))?;
    rawip.bind(&SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0).into())?;

    // mac layer creation
    let mut mac = MacLayer::new_with_default_phy(self_mac);
    let mut recv_fragments = Vec::<IpPackFrag>::new();
    let mut send_fragments = VecDeque::<IpPackFrag>::new();

    // socket in use: sock-addr <-> IPC socket
    let mut icmp_socks = Vec::<(Ipv4Addr, Socket)>::new();
    let mut tcp_socks = Vec::<(SocketAddrV4, Socket)>::new();
    let mut udp_socks = Vec::<(SocketAddrV4, Socket)>::new();

    // main loop
    loop {
      todo!()
    }
  }
}

use crate::{
  common::{RAWIP_PACK_SIZE, RAWSOCK_TIMEOUT},
  packet::{compose_icmp, compose_tcp, compose_udp, parse_icmp, parse_tcp, parse_udp, IpOverMac},
  ASockProtocol,
};
use pnet::packet::{
  ipv4::{Ipv4, Ipv4Packet, MutableIpv4Packet},
  FromPacket, MutablePacket, Packet,
};
use proj2_multiple_access::MacAddr;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use socket2::{Domain, Socket, Type};
use std::{
  collections::HashMap,
  io::{ErrorKind, Result},
  mem::{transmute, MaybeUninit},
  net::{Ipv4Addr, SocketAddrV4},
};

struct WrapRawSock {
  recv_buf: [MaybeUninit<u8>; RAWIP_PACK_SIZE],
  send_buf: [u8; RAWIP_PACK_SIZE],
  rawsock: Socket,
}

impl WrapRawSock {
  fn new() -> Result<Self> {
    // RAW IP packet socket creation
    let rawsock = Socket::new(Domain::IPV4, Type::RAW, None)?;
    rawsock.set_read_timeout(Some(RAWSOCK_TIMEOUT))?;
    rawsock.set_write_timeout(Some(RAWSOCK_TIMEOUT))?;
    rawsock.bind(&SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0).into())?;
    Ok(Self {
      recv_buf: [MaybeUninit::zeroed(); RAWIP_PACK_SIZE],
      send_buf: [0; RAWIP_PACK_SIZE],
      rawsock,
    })
  }
  fn recv(&mut self) -> Result<Ipv4> {
    let n = self.rawsock.recv(&mut self.recv_buf)?;
    let ipv4 = unsafe {
      let buf: [u8; RAWIP_PACK_SIZE] = transmute(self.recv_buf);
      let ipv4_pack = Ipv4Packet::new(&buf[..n]).ok_or(ErrorKind::InvalidData)?;
      ipv4_pack.from_packet()
    };
    Ok(ipv4)
  }
  fn send(&mut self, ipv4: Ipv4) -> Result<()> {
    let dest = SocketAddrV4::new(ipv4.destination, 0);
    let mut pack = MutableIpv4Packet::new(&mut self.send_buf).ok_or(ErrorKind::InvalidData)?;
    pack.populate(&ipv4);
    self.rawsock.send_to(pack.packet(), &dest.into())?;
    Ok(())
  }
}

/// - out: Athernet port -> Internet port
/// - in:  Internet port -> Athernet port
struct NatTable {
  anet2inet: HashMap<u16, u16>,
  inet2anet: HashMap<u16, u16>,
}

impl NatTable {
  fn new() -> Self {
    Self {
      anet2inet: Default::default(),
      inet2anet: Default::default(),
    }
  }
  fn anet_to_inet(&self, anet_port: u16) -> Option<u16> {
    self.anet2inet.get(&anet_port).and_then(|x| Some(*x))
  }
  fn inet_to_anet(&self, inet_port: u16) -> Option<u16> {
    self.inet2anet.get(&inet_port).and_then(|x| Some(*x))
  }
  fn add(&mut self, anet_port: u16, inet_port: u16) -> bool {
    if self.anet2inet.contains_key(&anet_port) || self.inet2anet.contains_key(&inet_port) {
      false
    } else {
      self.anet2inet.insert(anet_port, inet_port);
      self.inet2anet.insert(inet_port, anet_port);
      true
    }
  }
  fn remove(&mut self, anet_port: u16, inet_port: u16) -> bool {
    if self.anet_to_inet(anet_port) != Some(inet_port) || self.inet_to_anet(inet_port) != Some(anet_port) {
      false
    } else {
      self.anet2inet.remove(&anet_port);
      self.inet2anet.remove(&inet_port);
      true
    }
  }
}

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
///
/// NAT behavior:
///
/// - out: Athernet -> other LAN
///        1. add a mapping (source socket address <-> NAT port)
///        2. replace the source address with IP NAT address
///        3. replace the source port to a NAT port
///        4. re-compute the checksum for transport layer and IP layer
///        5. send via RAW socket
/// - in: other LAN -> Athernet
///        1. lookup the mapping to find internal socket address
///        2. replace the destination address with internal node IP address
///        3. replace the destination port with internal node port
///        4. re-compute the checksum for transport layer and IP layer
///        5. send via MAC
pub struct IpLayerGateway {
  // L2/L3 address
  self_ip: Ipv4Addr,
  peer_ip: Ipv4Addr,
  // send/recv IPv4 packets via MAC
  ip_txrx: IpOverMac,
  // raw socket for send/recv IPv4 packets
  rawsock: WrapRawSock,
  // NAT table:
  udp_nat: NatTable,
  tcp_nat: NatTable,
  icmp_nat: NatTable,
  // random number generator
  rng: ThreadRng,
}

impl IpLayerGateway {
  /// forward a IPv4 packet to other LAN
  fn forward_out(&mut self, ipv4: Ipv4) {
    match ipv4.next_level_protocol.try_into() {
      Ok(ASockProtocol::UDP) => {
        parse_udp(&ipv4).and_then(|mut udp| {
          // already have a mapping
          while self.udp_nat.anet_to_inet(udp.source).is_none() {
            self.udp_nat.add(udp.source, self.rng.gen());
          }
          // UDP NAT: change source port
          let inet_port = self.udp_nat.anet_to_inet(udp.source).unwrap();
          udp.source = inet_port;
          // UDP NAT: change source address
          let ipv4 = compose_udp(&udp, self.self_ip, ipv4.destination);
          // compose function should recompute checksum
          let _ = self.rawsock.send(ipv4);

          Some(())
        });
      }
      Ok(ASockProtocol::ICMP) => todo!(),
      Ok(ASockProtocol::TCP) => todo!(),
      Err(_) => todo!(),
    }
  }
  /// forward a IPv4 packet into Athernet
  fn forward_in(&mut self, ipv4: Ipv4) {
    match ipv4.next_level_protocol.try_into() {
      Ok(ASockProtocol::UDP) => {
        parse_udp(&ipv4).and_then(|mut udp| {
          if let Some(anet_port) = self.udp_nat.inet_to_anet(udp.destination) {
            // UDP NAT: change dest port
            udp.destination = anet_port;
            // UDP NAT: change dest port
            let ipv4 = compose_udp(&udp, ipv4.source, self.peer_ip);
            // compose function should recompute checksum
            let _ = self.rawsock.send(ipv4);
          }

          Some(())
        });
      }
      Ok(ASockProtocol::ICMP) => {
        // TODO: NAT in for ICMP
        return;
      }
      Ok(ASockProtocol::TCP) => {
        // TODO: NAT in for TCP
        return;
      }
      Err(_) => {
        // TODO: logging
        // other transport protocol, ignored
        return;
      }
    }
  }

  /// handle network traffic: Athernet -> other LAN
  fn handle_out(&mut self) {
    self.ip_txrx.send_poll();
    let maybe_ipv4 = self.ip_txrx.recv_poll();
    // on receiving IPv4 packet from peer: forward it to internet
    if let Some(ipv4) = maybe_ipv4 {
      self.forward_out(ipv4);
    }
  }
  /// handle network traffic: other LAN -> Athernet
  fn handle_in(&mut self) {
    if let Ok(ipv4) = self.rawsock.recv() {
      self.forward_in(ipv4);
    }
  }
  /// Run as a gateway node: NAT
  pub fn run(&mut self) {
    loop {
      self.handle_in();
      self.handle_out();
    }
  }

  /// Build IP layer on top of MAC layer.
  /// The MAC address and IP address should be given.
  ///
  /// - `self_addr`: the MAC address and IP address of current node
  /// - `peer_addr`: the MAC address and IP address of peer node
  pub fn new(self_addr: (MacAddr, Ipv4Addr), peer_addr: (MacAddr, Ipv4Addr)) -> Result<Self> {
    Ok(Self {
      self_ip: self_addr.1,
      peer_ip: peer_addr.1,
      ip_txrx: IpOverMac::new(self_addr.0, peer_addr.0),
      rawsock: WrapRawSock::new()?,
      udp_nat: NatTable::new(),
      tcp_nat: NatTable::new(),
      icmp_nat: NatTable::new(),
      rng: thread_rng(),
    })
  }
}
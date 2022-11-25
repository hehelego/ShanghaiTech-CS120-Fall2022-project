use crate::{
  common::{NAT_ICMP_BYPASS_PATTERN, RAWIP_PACK_SIZE, RAWSOCK_TIMEOUT},
  packet::{compose_icmp, compose_tcp, compose_udp, parse_icmp, parse_tcp, parse_udp, IpOverMac},
  ASockProtocol,
};
use pnet::packet::{
  icmp::{Icmp, IcmpCode, IcmpTypes},
  ipv4::{Ipv4, Ipv4Packet, MutableIpv4Packet},
  FromPacket, Packet,
};
use proj2_multiple_access::MacAddr;
use rand::Rng;
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
  inet_addr: Ipv4Addr,
  rawsock_udp: Socket,
  rawsock_icmp: Socket,
  rawsock_tcp: Socket,
}

impl WrapRawSock {
  fn rawsock_new(addr: Ipv4Addr, protocol: ASockProtocol) -> Result<Socket> {
    let rawsock = Socket::new(Domain::IPV4, Type::RAW, Some(protocol.into()))?;
    // Credit: https://stackoverflow.com/questions/33272644/raw-socket-unexpected-ip-header-added-when-sending-self-made-ip-tcp-packets
    rawsock.set_header_included(true)?;
    rawsock.set_read_timeout(Some(RAWSOCK_TIMEOUT))?;
    rawsock.set_write_timeout(Some(RAWSOCK_TIMEOUT))?;
    rawsock.bind(&SocketAddrV4::new(addr, 0).into())?;
    Ok(rawsock)
  }

  fn new(addr: Ipv4Addr) -> Result<Self> {
    Ok(Self {
      recv_buf: [MaybeUninit::zeroed(); RAWIP_PACK_SIZE],
      send_buf: [0; RAWIP_PACK_SIZE],
      inet_addr: addr,
      rawsock_udp: Self::rawsock_new(addr, ASockProtocol::UDP)?,
      rawsock_icmp: Self::rawsock_new(addr, ASockProtocol::ICMP)?,
      rawsock_tcp: Self::rawsock_new(addr, ASockProtocol::TCP)?,
    })
  }
  fn recv(&mut self) -> Result<Ipv4> {
    let n = self
      .rawsock_udp
      .recv(&mut self.recv_buf)
      .or_else(|_| self.rawsock_icmp.recv(&mut self.recv_buf))
      .or_else(|_| self.rawsock_tcp.recv(&mut self.recv_buf))?;

    let ipv4 = unsafe {
      let buf: [u8; RAWIP_PACK_SIZE] = transmute(self.recv_buf);
      let ipv4_pack = Ipv4Packet::new(&buf[..n]).ok_or(ErrorKind::InvalidData)?;
      ipv4_pack.from_packet()
    };
    assert_eq!(ipv4.destination, self.inet_addr);
    let protocol: Result<ASockProtocol> = ipv4.next_level_protocol.try_into();
    assert!(protocol.is_ok());
    Ok(ipv4)
  }
  fn send(&mut self, ipv4: Ipv4) -> Result<()> {
    assert_eq!(ipv4.source, self.inet_addr);

    let sock = match ipv4.next_level_protocol.try_into()? {
      ASockProtocol::UDP => &self.rawsock_udp,
      ASockProtocol::ICMP => &self.rawsock_icmp,
      ASockProtocol::TCP => &self.rawsock_tcp,
    };

    let len = ipv4.total_length as usize;
    let dest = SocketAddrV4::new(ipv4.destination, 0);
    let mut pack = MutableIpv4Packet::new(&mut self.send_buf[..len]).ok_or(ErrorKind::InvalidData)?;
    pack.populate(&ipv4);
    sock.send_to(pack.packet(), &dest.into())?;

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
  /// table lookup: Athernet port -> Internet port
  fn find_a2i(&self, anet_port: u16) -> Option<u16> {
    self.anet2inet.get(&anet_port).copied()
  }
  /// table lookup: Internet port -> Athernet port
  fn find_i2a(&self, inet_port: u16) -> Option<u16> {
    self.inet2anet.get(&inet_port).copied()
  }
  /// find a already existing mapping or add a new random mapping: A -> I
  fn find_or_add(&mut self, anet_port: u16) -> u16 {
    // already have a mapping
    if let Some(inet_port) = self.find_a2i(anet_port) {
      return inet_port;
    }
    // find an unused Internet port number
    let inet_port = rand::thread_rng()
      .sample_iter(rand::distributions::Standard)
      .find(|port| !self.inet2anet.contains_key(port))
      .unwrap();
    self.anet2inet.insert(anet_port, inet_port);
    self.inet2anet.insert(inet_port, anet_port);

    inet_port
  }
  /// remove a NAT port mapping
  /// TODO: remove a mapping if no packet send/recv in the last minute.
  fn remove(&mut self, anet_port: u16, inet_port: u16) -> bool {
    if self.find_a2i(anet_port) != Some(inet_port) || self.find_i2a(inet_port) != Some(anet_port) {
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
  anet_self_ip: Ipv4Addr,
  anet_peer_ip: Ipv4Addr,
  // send/recv IPv4 packets via MAC
  ip_txrx: IpOverMac,
  // raw socket for send/recv IPv4 packets: in Internet instead of Athernet
  rawsock: WrapRawSock,
  inet_self_ip: Ipv4Addr,
  // NAT table: Internet port <-> Athernet port
  nat: NatTable,
}

fn icmp_can_pass(icmp: &Icmp) -> bool {
  if icmp.icmp_type == IcmpTypes::EchoReply {
    true
  } else if icmp.icmp_type == IcmpTypes::EchoRequest {
    let payload_len = icmp.payload.len();
    let suffix_len = NAT_ICMP_BYPASS_PATTERN.len();
    &icmp.payload[payload_len - suffix_len..] == NAT_ICMP_BYPASS_PATTERN
  } else {
    false
  }
}

/// The destination of a packet send to gateway is
/// - either targeting a node within Athernet,
/// - or a node in the Internet.
enum DestNet {
  Athernet,
  Internet,
}

impl IpLayerGateway {
  /// determine whether we should forward the packet to Internet via NAT
  /// or route the packet in Athernet LAN
  fn pack_dest_net(&self, ipv4: &Ipv4) -> DestNet {
    if [self.anet_self_ip, self.anet_peer_ip].contains(&ipv4.destination) {
      DestNet::Athernet
    } else {
      DestNet::Internet
    }
  }

  /// forward a IPv4 packet to other LAN
  fn forward_out(&mut self, ipv4: Ipv4) {
    log::debug!("Athernet -> Internet: {:?} -> {:?}", ipv4.source, ipv4.destination);
    match ipv4.next_level_protocol.try_into() {
      Ok(ASockProtocol::UDP) => {
        if let Some(mut udp) = parse_udp(&ipv4) {
          // UDP NAT: change source port
          let inet_port = self.nat.find_or_add(udp.source);
          udp.source = inet_port;
          // UDP NAT: change source address
          let ipv4 = compose_udp(&udp, self.inet_self_ip, ipv4.destination);
          // compose function should recompute checksum
          let _ = self.rawsock.send(ipv4);

          log::debug!("forward A->I UDP, inet_port={}", inet_port);
        }
      }
      Ok(ASockProtocol::ICMP) => {
        if let Some(icmp) = parse_icmp(&ipv4) {
          // ICMP NAT: change source address
          let ipv4 = compose_icmp(&icmp, self.inet_self_ip, ipv4.destination);
          // compose function should recompute checksum
          let _ = self.rawsock.send(ipv4);

          log::debug!("forward A->I ICMP");
        }
      }
      Ok(ASockProtocol::TCP) => {
        if let Some(mut tcp) = parse_udp(&ipv4) {
          // TCP NAT: change source port
          let inet_port = self.nat.find_or_add(tcp.source);
          tcp.source = inet_port;
          // TCP NAT: change source address
          let ipv4 = compose_udp(&tcp, self.inet_self_ip, ipv4.destination);
          // compose function should recompute checksum
          let _ = self.rawsock.send(ipv4);

          log::debug!("forward A->I TCP, inet_port={}", inet_port);
        }
      }
      Err(_) => {
        log::debug!("forward A->I UNKNOWN, discard packet");
      }
    }
  }
  /// forward a IPv4 packet into Athernet
  fn forward_in(&mut self, ipv4: Ipv4) {
    log::debug!("Internet -> Athernet: {:?} -> {:?}", ipv4.source, ipv4.destination);
    match ipv4.next_level_protocol.try_into() {
      Ok(ASockProtocol::UDP) => {
        if let Some(mut udp) = parse_udp(&ipv4) {
          if let Some(anet_port) = self.nat.find_i2a(udp.destination) {
            // UDP NAT: change dest port
            udp.destination = anet_port;
            // UDP NAT: change dest address
            let ipv4 = compose_udp(&udp, ipv4.source, self.anet_peer_ip);
            // compose function should recompute checksum
            // send to peer via MAC
            self.ip_txrx.send(&ipv4);

            log::debug!("forward I->A UDP, anet_port={}", anet_port);
          }
        }
      }
      Ok(ASockProtocol::ICMP) => {
        if let Some(icmp) = parse_icmp(&ipv4) {
          if !icmp_can_pass(&icmp) {
            log::debug!("forward I->A ICMP cannot pass NAT, discard");
            return;
          }
          // ICMP NAT: change dest address
          let ipv4 = compose_icmp(&icmp, ipv4.source, self.anet_peer_ip);
          // compose function should recompute checksum
          // send to peer via MAC
          self.ip_txrx.send(&ipv4);
        }
      }
      Ok(ASockProtocol::TCP) => {
        if let Some(mut tcp) = parse_tcp(&ipv4) {
          if let Some(anet_port) = self.nat.find_i2a(tcp.destination) {
            // UDP NAT: change dest port
            tcp.destination = anet_port;
            // UDP NAT: change dest address
            let ipv4 = compose_tcp(&tcp, ipv4.source, self.anet_peer_ip);
            // compose function should recompute checksum
            // send to peer via MAC
            self.ip_txrx.send(&ipv4);

            log::debug!("forward I->A TCP, anet_port={}", anet_port);
          }
        }
      }
      Err(_) => {
        // other transport protocol, ignored
        log::debug!("forward I->A UNKNOWN, discard packet");
      }
    }
  }

  /// handle network traffic: Athernet -> other LAN
  fn handle_out(&mut self) {
    // try to receive IPv4 packet in Athernet
    let maybe_ipv4 = self.ip_txrx.recv_poll();
    // on receiving IPv4 packet from peer
    if let Some(ipv4) = maybe_ipv4 {
      log::debug!(
        "recv ipv4 from Athernet-MAC {:?} -> {:?}",
        ipv4.source,
        ipv4.destination
      );
      if ipv4.destination == self.anet_self_ip {
        log::debug!("last packet is targeting us");
        self.on_recv_ipv4(&ipv4);
        return;
      }
      match self.pack_dest_net(&ipv4) {
        // route the packet in Athernet
        DestNet::Athernet => {
          self.ip_txrx.send(&ipv4);
        }
        // forward it to Internet
        DestNet::Internet => {
          self.forward_out(ipv4);
        }
      }
    }
  }
  /// handle network traffic: other LAN -> Athernet
  fn handle_in(&mut self) {
    self.ip_txrx.send_poll();
    if let Ok(ipv4) = self.rawsock.recv() {
      log::debug!(
        "recv ipv4 from Internet-RAWSOCK {:?} -> {:?}, into forward I->A",
        ipv4.source,
        ipv4.destination
      );
      self.forward_in(ipv4);
    }
  }
  /// Run as a gateway node: NAT
  pub fn run(&mut self) {
    loop {
      log::trace!("gateway nat mainloop iteration");
      self.handle_in();
      self.handle_out();
    }
  }

  /// handle ICMP ping request message comming from another node
  fn handle_ping(&mut self, icmp: Icmp, from: Ipv4Addr) {
    if icmp.icmp_type == IcmpTypes::EchoRequest {
      log::debug!("response to ping request from {:?} in Athernet", from,);
      let icmp = Icmp {
        icmp_type: IcmpTypes::EchoReply,
        icmp_code: IcmpCode(0),
        checksum: 0,
        payload: icmp.payload,
      };
      let ipv4 = compose_icmp(&icmp, self.anet_self_ip, from);
      self.ip_txrx.send(&ipv4);
    }
  }
  /// called when receiving an ipv4 packet to us
  fn on_recv_ipv4(&mut self, ipv4: &Ipv4) {
    // handle ICMP echo request
    if let Ok(ASockProtocol::ICMP) = ipv4.next_level_protocol.try_into() {
      if let Some(icmp) = parse_icmp(ipv4) {
        self.handle_ping(icmp, ipv4.source);
      }
    }
    // TODO: allow gateway node to have Athernet transport layer
  }

  /// Build IP layer on top of MAC layer.
  /// The MAC address and IP address should be given.
  ///
  /// - `self_addr`: the MAC address and IP address of current node
  /// - `peer_addr`: the MAC address and IP address of peer node
  /// - `inet_addr`: address in the Internet
  pub fn new(self_addr: (MacAddr, Ipv4Addr), peer_addr: (MacAddr, Ipv4Addr), inet_addr: Ipv4Addr) -> Result<Self> {
    log::debug!(
      "starting IP layer for gateway@{:?}, internal@{:?}",
      self_addr,
      peer_addr
    );

    Ok(Self {
      anet_self_ip: self_addr.1,
      anet_peer_ip: peer_addr.1,
      ip_txrx: IpOverMac::new(self_addr.0, peer_addr.0),
      rawsock: WrapRawSock::new(inet_addr)?,
      inet_self_ip: inet_addr,
      nat: NatTable::new(),
    })
  }
}

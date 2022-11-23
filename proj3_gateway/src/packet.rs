use pnet::packet::{
  icmp::{checksum as icmp_checksum, *},
  ipv4::{checksum as ipv4_checksum, *},
  tcp::{ipv4_checksum as tcp_checksum, *},
  udp::{ipv4_checksum as udp_checksum, *},
  FromPacket, MutablePacket, Packet,
};
use proj2_multiple_access::{MacAddr, MacLayer};

use std::{collections::VecDeque, net::Ipv4Addr};

/// try to extract an ICMP packet from the payload of an IPv4 packet.
pub(crate) fn parse_icmp(ipv4: &Ipv4) -> Option<Icmp> {
  IcmpPacket::new(&ipv4.payload).map(|packet| packet.from_packet())
}
/// try to extract an UDP packet from the payload of an IPv4 packet.
pub(crate) fn parse_udp(ipv4: &Ipv4) -> Option<Udp> {
  UdpPacket::new(&ipv4.payload).map(|packet| packet.from_packet())
}
/// try to extract a TCP packet from the payload of an IPv4 packet.
pub(crate) fn parse_tcp(ipv4: &Ipv4) -> Option<Tcp> {
  TcpPacket::new(&ipv4.payload).map(|packet| packet.from_packet())
}

pub(crate) fn compose_ipv4(src: Ipv4Addr, dest: Ipv4Addr, next_level: &[u8]) -> Ipv4 {
  // 20 for IPv4 header with no extra options
  let mut buf = vec![0; next_level.len() + 20];
  let mut ip_pack = MutableIpv4Packet::new(&mut buf).unwrap();
  ip_pack.set_total_length(20 + next_level.len() as u16);
  ip_pack.set_source(src);
  ip_pack.set_destination(dest);
  ip_pack.set_payload(next_level);
  ip_pack.set_checksum(ipv4_checksum(&ip_pack.to_immutable()));
  ip_pack.from_packet()
}

/// compose an IPv4 packet which encapsulates an ICMP packet
pub(crate) fn compose_icmp(icmp: &Icmp, src: Ipv4Addr, dest: Ipv4Addr) -> Ipv4 {
  // 8: ICMP header.
  let mut buf = vec![0; icmp.payload.len() + 8];
  let mut icmp_pack = MutableIcmpPacket::new(&mut buf).unwrap();
  icmp_pack.populate(&icmp);
  icmp_pack.set_checksum(icmp_checksum(&icmp_pack.to_immutable()));
  compose_ipv4(src, dest, icmp_pack.packet())
}

/// compose an IPv4 packet which encapsulates an UDP packet
pub(crate) fn compose_udp(udp: &Udp, src: Ipv4Addr, dest: Ipv4Addr) -> Ipv4 {
  // 8: UDP header.
  let mut buf = vec![0; udp.payload.len() + 8];
  let mut udp_pack = MutableUdpPacket::new(&mut buf).unwrap();
  udp_pack.populate(&udp);
  udp_pack.set_checksum(udp_checksum(&udp_pack.to_immutable(), &src, &dest));
  compose_ipv4(src, dest, udp_pack.packet())
}

/// compose an IPv4 packet which encapsulates an TCP packet
pub(crate) fn compose_tcp(tcp: &Tcp, src: Ipv4Addr, dest: Ipv4Addr) -> Ipv4 {
  // 20: TCP header, with no extra options. 4 bit per option.
  let mut buf = vec![0; 20 + tcp.payload.len() + 4 * tcp.data_offset as usize];
  let mut tcp_pack = MutableTcpPacket::new(&mut buf).unwrap();
  tcp_pack.populate(&tcp);
  tcp_pack.set_checksum(tcp_checksum(&tcp_pack.to_immutable(), &src, &dest));
  compose_ipv4(src, dest, tcp_pack.packet())
}

/// fragments of an IP packet, can be send directly to peer via MAC layer
pub(crate) struct IpPackFrag {
  data: Vec<u8>,
  pub last: bool,
}

impl IpPackFrag {
  /// maximum data size per fragment:
  /// - [0..N-1]: data chunk
  /// - [N-1]: is last fragment
  const FRAG_SIZE: usize = MacLayer::MTU - 1;
  /// parse a fragment from a MAC packet payload
  pub(crate) fn from_mac_payload(mac_payload: &[u8]) -> Self {
    let (last, content) = mac_payload.split_last().unwrap();
    let last = *last == 1;
    let content = content.to_vec();
    Self { data: content, last }
  }
  /// populate a MAC packet payload with a fragment
  pub(crate) fn into_mac_payload(self) -> Vec<u8> {
    let Self { mut data, last } = self;
    data.push(last as u8);
    data
  }
}

/// Split a IPv4 packet into chunks so that the packet can be send over MAC layer
fn fragment_ipv4(ipv4: &Ipv4) -> Vec<IpPackFrag> {
  let mut buf = vec![0; ipv4.total_length as usize];
  let mut pack = MutableIpv4Packet::new(&mut buf).unwrap();
  pack.populate(&ipv4);

  let mut fragments: Vec<_> = buf
    .chunks(IpPackFrag::FRAG_SIZE)
    .map(|chunk| IpPackFrag {
      data: Vec::from(chunk),
      last: false,
    })
    .collect();
  fragments.last_mut().unwrap().last = true;

  fragments
}

/// Reassemble an IPv4 packet from bytes
fn reassemble_ipv4(bytes: impl Iterator<Item = u8>) -> Ipv4 {
  let buf: Vec<_> = bytes.collect();
  let packet = Ipv4Packet::new(&buf).unwrap();

  packet.from_packet()
}

/// A wrapper of MAC layer object: sending/receiving IP packets via MAC.
/// - Split an IPv4 packet into multiple fragments and send them to peer
/// - Reassemble an IPv4 packet from multiple received fragments
pub(crate) struct IpOverMac {
  mac: MacLayer,
  self_addr: MacAddr,
  peer_addr: MacAddr,
  recv_frags: Vec<u8>,
  send_frags: VecDeque<IpPackFrag>,
}

impl IpOverMac {
  pub fn new(self_addr: MacAddr, peer_addr: MacAddr) -> Self {
    Self {
      mac: MacLayer::new_with_default_phy(self_addr),
      self_addr,
      peer_addr,
      recv_frags: Default::default(),
      send_frags: Default::default(),
    }
  }
  /// schedule to send a packet
  pub fn send(&mut self, ipv4: &Ipv4) {
    self.send_frags.extend(fragment_ipv4(ipv4));
  }
  /// Called every iteration.
  /// Send a fragment to peer
  pub fn send_poll(&mut self) {
    if let Some(frag) = self.send_frags.pop_front() {
      self.mac.send_to(self.peer_addr, frag.into_mac_payload());
    }
  }
  /// Called every iteration.
  /// Try to receive a fragment then reassemble a IPv4 packet if it is possible
  pub fn recv_poll(&mut self) -> Option<Ipv4> {
    if let Some(frag) = self.mac.try_recv() {
      let frag = IpPackFrag::from_mac_payload(&frag);
      let last = frag.last;
      self.recv_frags.extend(frag.data);
      if last {
        let ipv4 = reassemble_ipv4(self.recv_frags.drain(..));
        return Some(ipv4);
      }
    }
    None
  }
}

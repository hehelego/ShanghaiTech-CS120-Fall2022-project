use pnet::packet::{
  icmp::{checksum as icmp_checksum, *},
  ipv4::{checksum as ipv4_checksum, *},
  tcp::{ipv4_checksum as tcp_checksum, *},
  udp::{ipv4_checksum as udp_checksum, *},
  FromPacket, Packet,
};

use std::net::Ipv4Addr;

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

fn compose_ipv4(src: Ipv4Addr, dest: Ipv4Addr, next_level: &[u8]) -> Ipv4 {
  // 20 for IPv4 header with no extra options
  let mut buf = vec![0; next_level.len() + 20];
  let mut ip_pack = MutableIpv4Packet::new(&mut buf).unwrap();
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
  // 20: TCP header, with no extra options.
  let mut buf = vec![0; tcp.payload.len() + 8];
  let mut tcp_pack = MutableTcpPacket::new(&mut buf).unwrap();
  tcp_pack.populate(&tcp);
  tcp_pack.set_checksum(tcp_checksum(&tcp_pack.to_immutable(), &src, &dest));
  compose_ipv4(src, dest, tcp_pack.packet())
}

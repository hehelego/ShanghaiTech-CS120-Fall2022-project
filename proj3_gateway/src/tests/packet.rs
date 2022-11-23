use std::net::Ipv4Addr;

use crate::packet::{compose_icmp, compose_ipv4, compose_tcp, compose_udp, parse_icmp, parse_tcp, parse_udp};
use pnet::packet::{
  icmp::{Icmp, IcmpCode, IcmpTypes},
  tcp::Tcp,
  udp::Udp,
};
use rand::{distributions::Standard, Rng};

const TEST: usize = 100;

fn ipv4_once() {
  let mut rng = rand::thread_rng();
  let len = rng.gen_range(8..100);

  let src = Ipv4Addr::new(0, 1, 2, 3);
  let dest = Ipv4Addr::new(0, 1, 2, 3);

  let data: Vec<_> = rng.sample_iter(Standard).take(len as usize).collect();
  let ipv4 = compose_ipv4(src, dest, &data, crate::ASockProtocol::UDP);
  assert_eq!(data, ipv4.payload);
}

fn udp_once() {
  let mut rng = rand::thread_rng();
  let len = rng.gen_range(2..100);

  let udp = Udp {
    source: rng.gen(),
    destination: rng.gen(),
    length: len,
    checksum: 0,
    payload: rng.sample_iter(Standard).take(len as usize).collect(),
  };

  let src = Ipv4Addr::new(0, 1, 2, 3);
  let dest = Ipv4Addr::new(7, 6, 5, 4);

  let ipv4 = compose_udp(&udp, src, dest);
  let udp_out = parse_udp(&ipv4).unwrap();

  assert_eq!(udp.source, udp_out.source);
  assert_eq!(udp.destination, udp_out.destination);
  assert_eq!(udp.length, udp_out.length);
  assert_eq!(udp.payload, udp_out.payload);
}
fn tcp_once() {
  let mut rng = rand::thread_rng();
  let len = rng.gen_range(2..100);

  let tcp = Tcp {
    source: rng.gen(),
    destination: rng.gen(),
    sequence: rng.gen(),
    acknowledgement: rng.gen(),
    data_offset: 0,
    reserved: 0,
    flags: 0,
    window: rng.gen(),
    checksum: 0,
    urgent_ptr: 0,
    options: vec![],
    payload: rng.sample_iter(Standard).take(len as usize).collect(),
  };

  let src = Ipv4Addr::new(0, 1, 2, 3);
  let dest = Ipv4Addr::new(7, 6, 5, 4);

  let ipv4 = compose_tcp(&tcp, src, dest);
  let tcp_out = parse_tcp(&ipv4).unwrap();

  assert_eq!(tcp.source, tcp_out.source);
  assert_eq!(tcp.destination, tcp_out.destination);
  assert_eq!(tcp.sequence, tcp_out.sequence);
  assert_eq!(tcp.data_offset, tcp_out.data_offset);
  assert_eq!(tcp.reserved, tcp_out.reserved);
  assert_eq!(tcp.flags, tcp_out.flags);
  assert_eq!(tcp.window, tcp_out.window);
  assert_eq!(tcp.urgent_ptr, tcp_out.urgent_ptr);
  assert_eq!(tcp.payload, tcp_out.payload);
}
fn icmp_once() {
  let mut rng = rand::thread_rng();
  let len = rng.gen_range(4..100);

  let icmp = Icmp {
    icmp_type: IcmpTypes::EchoReply,
    icmp_code: IcmpCode(rng.gen()),
    checksum: 0,
    payload: rng.sample_iter(Standard).take(len as usize).collect(),
  };

  let src = Ipv4Addr::new(0, 1, 2, 3);
  let dest = Ipv4Addr::new(7, 6, 5, 4);

  let ipv4 = compose_icmp(&icmp, src, dest);
  let icmp_out = parse_icmp(&ipv4).unwrap();

  assert_eq!(icmp.icmp_type, icmp_out.icmp_type);
  assert_eq!(icmp.icmp_code, icmp_out.icmp_code);
  assert_eq!(icmp.payload, icmp_out.payload[..len]);
}

#[test]
fn ipv4_pack_unpack() {
  for _ in 0..TEST {
    ipv4_once();
  }
}
#[test]
fn udp_pack_unpack() {
  for _ in 0..TEST {
    udp_once();
  }
}
#[test]
fn tcp_pack_unpack() {
  for _ in 0..TEST {
    tcp_once();
  }
}
#[test]
fn icmp_pack_unpack() {
  for _ in 0..TEST {
    icmp_once();
  }
}

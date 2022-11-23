use std::net::{Ipv4Addr, SocketAddrV4};

use proj3_gateway::UdpSocket;

fn main() {
  let self_addr: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 2), 3120);
  let dest_addr: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 18, 170), 3120);
  let upd_socket = UdpSocket::bind(self_addr).unwrap();
  let buf = "abcdefghijklmnopqrst".as_bytes();
  upd_socket.send_to(buf, dest_addr).unwrap();
}

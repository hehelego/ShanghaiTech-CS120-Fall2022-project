use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};

fn main() {
  let self_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 3120);
  let udp_socket = UdpSocket::bind(self_addr).unwrap();
  let mut buf: [u8; 30] = [0; 30];
  let (bytes, _) = udp_socket.recv_from(&mut buf).unwrap();
  println!("{}: {:?}", bytes, &buf[0..bytes]);
}

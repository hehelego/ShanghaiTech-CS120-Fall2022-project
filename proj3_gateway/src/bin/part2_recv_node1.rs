use clap::Parser;
use proj3_gateway::UdpSocket;
use std::{
  fs::File,
  io::Write,
  net::{Ipv4Addr, SocketAddrV4},
};

#[derive(Parser)]
struct Cli {
  /// host port
  port: u16,
}
fn main() {
  const SELF_ADDR: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 2);

  let port = Cli::parse().port;
  let self_addr = SocketAddrV4::new(SELF_ADDR, port);
  let udp_socket = UdpSocket::bind(self_addr).unwrap();
  let mut output_file = File::create("OUTPUT.txt").unwrap();
  let mut blank_line: bool = false;
  loop {
    if let Ok((data, src_dest)) = udp_socket.recv_from() {
      let size = data.len();
      println!("Receive {} bytes from {}", size, src_dest);
      if size == 0 {
        if blank_line {
          println!("Finish receiving.");
          break;
        }
        blank_line = true;
      } else {
        if blank_line {
          blank_line = false;
          writeln!(output_file).unwrap();
        }
        writeln!(output_file, "{}", String::from_utf8(data).unwrap()).unwrap();
      }
    }
  }
}
